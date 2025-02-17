package guardiand

import (
	"context"
	"encoding/hex"
	"fmt"
	publicrpcv1 "github.com/certusone/wormhole/node/pkg/proto/publicrpc/v1"
	"github.com/certusone/wormhole/node/pkg/vaa"
	"github.com/davecgh/go-spew/spew"
	"github.com/spf13/pflag"
	"io/ioutil"
	"log"
	"strconv"
	"strings"
	"time"

	"github.com/spf13/cobra"
	"github.com/status-im/keycard-go/hexutils"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/encoding/prototext"

	nodev1 "github.com/certusone/wormhole/node/pkg/proto/node/v1"
)

var (
	clientSocketPath *string
)

func init() {
	// Shared flags for all admin commands
	pf := pflag.NewFlagSet("commonAdminFlags", pflag.ContinueOnError)
	clientSocketPath = pf.String("socket", "", "gRPC admin server socket to connect to")
	err := cobra.MarkFlagRequired(pf, "socket")
	if err != nil {
		panic(err)
	}

	AdminClientInjectGuardianSetUpdateCmd.Flags().AddFlagSet(pf)
	AdminClientFindMissingMessagesCmd.Flags().AddFlagSet(pf)
	AdminClientListNodes.Flags().AddFlagSet(pf)
	DumpVAAByMessageID.Flags().AddFlagSet(pf)

	AdminCmd.AddCommand(AdminClientInjectGuardianSetUpdateCmd)
	AdminCmd.AddCommand(AdminClientFindMissingMessagesCmd)
	AdminCmd.AddCommand(AdminClientGovernanceVAAVerifyCmd)
	AdminCmd.AddCommand(AdminClientListNodes)
	AdminCmd.AddCommand(DumpVAAByMessageID)
}

var AdminCmd = &cobra.Command{
	Use:   "admin",
	Short: "Guardian node admin commands",
}

var AdminClientInjectGuardianSetUpdateCmd = &cobra.Command{
	Use:   "governance-vaa-inject [FILENAME]",
	Short: "Inject and sign a governance VAA from a prototxt file (see docs!)",
	Run:   runInjectGovernanceVAA,
	Args:  cobra.ExactArgs(1),
}

var AdminClientFindMissingMessagesCmd = &cobra.Command{
	Use:   "find-missing-messages [CHAIN_ID] [EMITTER_ADDRESS_HEX]",
	Short: "Find sequence number gaps for the given chain ID and emitter address",
	Run:   runFindMissingMessages,
	Args:  cobra.ExactArgs(2),
}

var DumpVAAByMessageID = &cobra.Command{
	Use:   "dump-vaa-by-message-id [MESSAGE_ID]",
	Short: "Retrieve a VAA by message ID (chain/emitter/seq) and decode and dump the VAA",
	Run:   runDumpVAAByMessageID,
	Args:  cobra.ExactArgs(1),
}

func getAdminClient(ctx context.Context, addr string) (*grpc.ClientConn, error, nodev1.NodePrivilegedServiceClient) {
	conn, err := grpc.DialContext(ctx, fmt.Sprintf("unix:///%s", addr), grpc.WithInsecure())

	if err != nil {
		log.Fatalf("failed to connect to %s: %v", addr, err)
	}

	c := nodev1.NewNodePrivilegedServiceClient(conn)
	return conn, err, c
}

func getPublicRPCServiceClient(ctx context.Context, addr string) (*grpc.ClientConn, error, publicrpcv1.PublicRPCServiceClient) {
	conn, err := grpc.DialContext(ctx, fmt.Sprintf("unix:///%s", addr), grpc.WithInsecure())

	if err != nil {
		log.Fatalf("failed to connect to %s: %v", addr, err)
	}

	c := publicrpcv1.NewPublicRPCServiceClient(conn)
	return conn, err, c
}

func runInjectGovernanceVAA(cmd *cobra.Command, args []string) {
	path := args[0]
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	conn, err, c := getAdminClient(ctx, *clientSocketPath)
	defer conn.Close()
	if err != nil {
		log.Fatalf("failed to get admin client: %v", err)
	}

	b, err := ioutil.ReadFile(path)
	if err != nil {
		log.Fatalf("failed to read file: %v", err)
	}

	var msg nodev1.InjectGovernanceVAARequest
	err = prototext.Unmarshal(b, &msg)
	if err != nil {
		log.Fatalf("failed to deserialize: %v", err)
	}

	resp, err := c.InjectGovernanceVAA(ctx, &msg)
	if err != nil {
		log.Fatalf("failed to submit governance VAA: %v", err)
	}

	for _, digest := range resp.Digests {
		log.Printf("VAA successfully injected with digest %s", hexutils.BytesToHex(digest))
	}
}

func runFindMissingMessages(cmd *cobra.Command, args []string) {
	chainID, err := strconv.Atoi(args[0])
	if err != nil {
		log.Fatalf("invalid chain ID: %v", err)
	}
	emitterAddress := args[1]

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	conn, err, c := getAdminClient(ctx, *clientSocketPath)
	defer conn.Close()
	if err != nil {
		log.Fatalf("failed to get admin client: %v", err)
	}

	msg := nodev1.FindMissingMessagesRequest{
		EmitterChain:   uint32(chainID),
		EmitterAddress: emitterAddress,
	}
	resp, err := c.FindMissingMessages(ctx, &msg)
	if err != nil {
		log.Fatalf("failed to run find FindMissingMessages RPC: %v", err)
	}

	for _, id := range resp.MissingMessages {
		fmt.Println(id)
	}

	log.Printf("processed %s sequences %d to %d (%d gaps)",
		emitterAddress, resp.FirstSequence, resp.LastSequence, len(resp.MissingMessages))
}

// runDumpVAAByMessageID uses GetSignedVAA to request the given message,
// then decode and dump the VAA.
func runDumpVAAByMessageID(cmd *cobra.Command, args []string) {
	// Parse the {chain,emitter,seq} string.
	parts := strings.Split(args[0], "/")
	if len(parts) != 3 {
		log.Fatalf("invalid message ID: %s", args[0])
	}
	chainID, err := strconv.ParseUint(parts[0], 10, 32)
	if err != nil {
		log.Fatalf("invalid chain ID: %v", err)
	}
	emitterAddress := parts[1]
	seq, err := strconv.ParseUint(parts[2], 10, 64)
	if err != nil {
		log.Fatalf("invalid sequence number: %v", err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	conn, err, c := getPublicRPCServiceClient(ctx, *clientSocketPath)
	defer conn.Close()
	if err != nil {
		log.Fatalf("failed to get public RPC service client: %v", err)
	}

	msg := publicrpcv1.GetSignedVAARequest{
		MessageId: &publicrpcv1.MessageID{
			EmitterChain:   publicrpcv1.ChainID(chainID),
			EmitterAddress: emitterAddress,
			Sequence:       seq,
		},
	}
	resp, err := c.GetSignedVAA(ctx, &msg)
	if err != nil {
		log.Fatalf("failed to run GetSignedVAA RPC: %v", err)
	}

	v, err := vaa.Unmarshal(resp.VaaBytes)
	if err != nil {
		log.Fatalf("failed to decode VAA: %v", err)
	}

	log.Printf("VAA with digest %s: %+v\n", v.HexDigest(), spew.Sdump(v))
	fmt.Printf("Bytes:\n%s\n", hex.EncodeToString(resp.VaaBytes))
}
