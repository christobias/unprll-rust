// Version
pub static VERSION: &str = "v1.0.0";
pub static RELEASE_NAME: &str = "Rusty Rabbit";

// pub const CRYPTONOTE_DNS_TIMEOUT_MS: u32 = 20000;
/*
pub const CRYPTONOTE_MAX_BLOCK_NUMBER = 500000000;
pub const CRYPTONOTE_GETBLOCKTEMPLATE_MAX_BLOCK_SIZE = 196608     // Size of block (bytes) that is the maximum that miners will produce
pub const CRYPTONOTE_MAX_TX_SIZE = 1000000000
pub const CRYPTONOTE_PUBLIC_ADDRESS_TEXTBLOB_VER = 0
pub const CRYPTONOTE_MINED_MONEY_UNLOCK_WINDOW_V1 = 3  // Block spans of 4 blocks = 3*4 = 12 blocks
pub const CRYPTONOTE_MINED_MONEY_UNLOCK_WINDOW_V2 = 4  // Block spans of 16 blocks = 4*16 = 64 blocks
pub const CURRENT_TRANSACTION_VERSION =    2
pub const CURRENT_BLOCK_MAJOR_VERSION =    1
pub const CURRENT_BLOCK_MINOR_VERSION =    9
pub const CRYPTONOTE_BLOCK_FUTURE_TIME_LIMIT_V1 =   330
pub const CRYPTONOTE_BLOCK_FUTURE_TIME_LIMIT_V2 =   70
pub const CRYPTONOTE_DEFAULT_TX_SPENDABLE_AGE_V1 =  2 // Blocks
pub const CRYPTONOTE_DEFAULT_TX_SPENDABLE_AGE_V2 =  10

pub const BLOCKCHAIN_TIMESTAMP_CHECK_WINDOW =       11

// MONEY_SUPPLY - total number coins to be generated
pub const MONEY_SUPPLY =      ((uint64_t)(-1))
pub const EMISSION_SPEED_FACTOR_PER_MINUTE =        (20)
pub const FINAL_SUBSIDY_PER_MINUTE  ((uint64_t)600000000000) // 6 * pow(10, 11)

pub const CRYPTONOTE_REWARD_BLOCKS_WINDOW =         100
pub const CRYPTONOTE_BLOCK_GRANTED_FULL_REWARD_ZONE       300000 //size of block (bytes) after which reward for block calculated using block size - second change, from v5
pub const CRYPTONOTE_COINBASE_BLOB_RESERVED_SIZE =  600
pub const CRYPTONOTE_DISPLAY_DECIMAL_POINT =        12
// COIN - number of smallest units in one coin
pub const COIN =     ((uint64_t)1000000000000) // pow(10, 12)

pub const FEE_PER_KB =        ((uint64_t)2000000000) // 2 * pow(10, 9)
pub const FEE_PER_BYTE =      ((uint64_t)300000)
pub const DYNAMIC_FEE_PER_KB_BASE_FEE =    ((uint64_t)2000000000) // 2 * pow(10,9)
pub const DYNAMIC_FEE_PER_KB_BASE_BLOCK_REWARD =    ((uint64_t)10000000000000) // 10 * pow(10,12)
pub const DYNAMIC_FEE_REFERENCE_TRANSACTION_WEIGHT        ((uint64_t)3000)

pub const ORPHANED_BLOCKS_MAX_COUNT 100

pub const DIFFICULTY_TARGET_V1      600  // seconds
pub const DIFFICULTY_TARGET_V2      120
pub const DIFFICULTY_WINDOW = 60
pub const DIFFICULTY_BLOCKS_COUNT   DIFFICULTY_WINDOW + 1

pub const CRYPTONOTE_LOCKED_TX_ALLOWED_DELTA_BLOCKS       1

pub const DIFFICULTY_BLOCKS_ESTIMATE_TIMESPAN =     DIFFICULTY_TARGET_V1 //just alias; used by tests

pub const BLOCKS_IDS_SYNCHRONIZING_DEFAULT_COUNT =  10000  //by default, blocks ids count in synchronizing
pub const BLOCKS_SYNCHRONIZING_DEFAULT_COUNT =      20     //by default, blocks count in blocks downloading

pub const CRYPTONOTE_MEMPOOL_TX_LIVETIME = (86400*3) //seconds, three days
pub const CRYPTONOTE_MEMPOOL_TX_FROM_ALT_BLOCK_LIVETIME   604800 //seconds, one week

pub const COMMAND_RPC_GET_BLOCKS_FAST_MAX_COUNT =   1000

pub const P2P_LOCAL_WHITE_PEERLIST_LIMIT = 1000
pub const P2P_LOCAL_GRAY_PEERLIST_LIMIT =  5000

pub const P2P_DEFAULT_CONNECTIONS_COUNT =  16
pub const P2P_DEFAULT_HANDSHAKE_INTERVAL = 60 =   //seconds
pub const P2P_DEFAULT_PACKET_MAX_SIZE =    50000000     //50000000 bytes maximum packet size
pub const P2P_DEFAULT_PEERS_IN_HANDSHAKE = 250
pub const P2P_DEFAULT_CONNECTION_TIMEOUT = 5000       //5 seconds
pub const P2P_DEFAULT_PING_CONNECTION_TIMEOUT =     2000       //2 seconds
pub const P2P_DEFAULT_INVOKE_TIMEOUT =     60*2*1000  //2 minutes
pub const P2P_DEFAULT_HANDSHAKE_INVOKE_TIMEOUT =    5000       //5 seconds
pub const P2P_DEFAULT_WHITELIST_CONNECTIONS_PERCENT       70
pub const P2P_DEFAULT_ANCHOR_CONNECTIONS_COUNT =    2

pub const P2P_FAILED_ADDR_FORGET_SECONDS = (60*60)     //1 hour
pub const P2P_IP_BLOCKTIME = (60*60*24)  //24 hour
pub const P2P_IP_FAILS_BEFORE_BLOCK 10
pub const P2P_IDLE_CONNECTION_KILL_INTERVAL =      (5*60) //5 minutes

pub const P2P_SUPPORT_FLAG_FLUFFY_BLOCKS = 0x01
pub const P2P_SUPPORT_FLAGS = P2P_SUPPORT_FLAG_FLUFFY_BLOCKS

pub const ALLOW_DEBUG_COMMANDS

pub const CRYPTONOTE_NAME   "unprll"
pub const CRYPTONOTE_POOLDATA_FILENAME =    "poolstate.bin"
pub const CRYPTONOTE_BLOCKCHAINDATA_FILENAME      "data.mdb"
pub const CRYPTONOTE_BLOCKCHAINDATA_LOCK_FILENAME "lock.mdb"
pub const P2P_NET_DATA_FILENAME =  "p2pstate.bin"
pub const MINER_CONFIG_FILE_NAME = "miner_conf.json"

pub const THREAD_STACK_SIZE 5 * 1024 * 1024

pub const HF_VERSION_DYNAMIC_FEE = 4
pub const HF_VERSION_MIN_MIXIN_4 = 6
pub const HF_VERSION_MIN_MIXIN_6 = 7
pub const HF_VERSION_ENFORCE_RCT = 6
pub const HF_VERSION_PER_BYTE_FEE =         8
pub const HF_VERSION_MIN_MIXIN_12 =         8
pub const HF_VERSION_BLOCK_TIME_REDUCTION = 11

pub const PER_KB_FEE_QUANTIZATION_DECIMALS        8

pub const HASH_OF_HASHES_STEP =    256

pub const DEFAULT_TXPOOL_MAX_WEIGHT =       648000000ull // 3 days at 300000, in bytes

pub const BULLETPROOF_MAX_OUTPUTS =         16
*/

/*
// New constants are intended to go here
namespace config
{
  uint64_t const DEFAULT_FEE_ATOMIC_XMR_PER_KB = 500; // Just a placeholder!  Change me!
  uint8_t const FEE_CALCULATION_MAX_RETRIES = 10;
  uint64_t const DEFAULT_DUST_THRESHOLD = ((uint64_t)2000000000); // 2 * pow(10, 9)
  uint64_t const BASE_REWARD_CLAMP_THRESHOLD = ((uint64_t)100000000); // pow(10, 8)
  std::string const P2P_REMOTE_DEBUG_TRUSTED_PUB_KEY = "0000000000000000000000000000000000000000000000000000000000000000";

  uint64_t const CRYPTONOTE_PUBLIC_ADDRESS_BASE58_PREFIX = 0x145023; // UNP
  uint64_t const CRYPTONOTE_PUBLIC_INTEGRATED_ADDRESS_BASE58_PREFIX = 0x291023; // UNPi
  uint64_t const CRYPTONOTE_PUBLIC_SUBADDRESS_BASE58_PREFIX = 0x211023; // UNPS
  uint16_t const P2P_DEFAULT_PORT = 21149;
  uint16_t const RPC_DEFAULT_PORT = 21150;
  uint16_t const ZMQ_RPC_DEFAULT_PORT = 21151;
  boost::uuids::uuid const NETWORK_ID = { {
      0x01 ,0x23, 0x45, 0x67 , 0x89, 0xAB , 0xCD, 0xEF, 0xED, 0xCB, 0xA9, 0x87, 0x65, 0x43, 0x21, 0x0F
    } }; // Bender's nightmare
  std::string const GENESIS_TX = "010301ff000180b8ceedf7ff03029b2e4c0281c0b02e7c53291a94d1d0cbff8883f8024f5142ee494ffbbd08807121017767aafcde9be00dcfd098715ebcf7f410daebc582fda69d24a28e9d0bc890d1";

  // Unprll specific
  uint64_t const HASH_CHECKPOINT_STEP_V1 = 10; // hashes between each checkpoint
  uint64_t const HASH_CHECKPOINT_STEP_V2 = 30;
  double const BLOCK_VALID_THRESHOLD = 0.10; // 10% of all hash checkpoints must verify correctly

  // Dandelion config
  uint8_t const DANDELION_TX_EMBARGO_PERIOD = 30;
  uint8_t const DANDELION_TX_STEM_PROPAGATION_PROBABILITY = 90;

  uint8_t const UNLOCK_DELTA_BLOCK_SPANS_V1 = 4;
  uint8_t const UNLOCK_DELTA_BLOCK_SPANS_V2 = 16;

  uint16_t const MAXIMUM_REQUESTS_PER_MINUTE = 5;

  namespace testnet
  {
    uint64_t const CRYPTONOTE_PUBLIC_ADDRESS_BASE58_PREFIX = 0x219023; // UNPT
    uint64_t const CRYPTONOTE_PUBLIC_INTEGRATED_ADDRESS_BASE58_PREFIX = 0x15e15023; // UNPTi
    uint64_t const CRYPTONOTE_PUBLIC_SUBADDRESS_BASE58_PREFIX = 0x3415023; // UNPTS
    uint16_t const P2P_DEFAULT_PORT = 21152;
    uint16_t const RPC_DEFAULT_PORT = 21153;
    uint16_t const ZMQ_RPC_DEFAULT_PORT = 21154;
    boost::uuids::uuid const NETWORK_ID = { {
        0x73 ,0x57, 0x45, 0x67 , 0x89, 0xAB , 0xCD, 0xEF, 0xED, 0xCB, 0xA9, 0x87, 0x65, 0x43, 0x21, 0x0F
      } }; // Bender's daydream
    std::string const GENESIS_TX = "010301ff000180b8ceedf7ff03029b2e4c0281c0b02e7c53291a94d1d0cbff8883f8024f5142ee494ffbbd0880712101168d0c4ca86fb55a4cf6a36d31431be1c53a3bd7411bb24e8832410289fa6f3b";
  }

  namespace stagenet
  {
    uint64_t const CRYPTONOTE_PUBLIC_ADDRESS_BASE58_PREFIX = 24;
    uint64_t const CRYPTONOTE_PUBLIC_INTEGRATED_ADDRESS_BASE58_PREFIX = 25;
    uint64_t const CRYPTONOTE_PUBLIC_SUBADDRESS_BASE58_PREFIX = 36;
    uint16_t const P2P_DEFAULT_PORT = 38080;
    uint16_t const RPC_DEFAULT_PORT = 38081;
    uint16_t const ZMQ_RPC_DEFAULT_PORT = 38082;
    boost::uuids::uuid const NETWORK_ID = { {
        0x12 ,0x30, 0xF1, 0x71 , 0x61, 0x04 , 0x41, 0x61, 0x17, 0x31, 0x00, 0x82, 0x16, 0xA1, 0xA1, 0x12
      } }; // Bender's daydream
    std::string const GENESIS_TX = "010301ff000180b8ceedf7ff0302df5d56da0c7d643ddd1ce61901c7bdc5fb1738bfe39fbe69c28a3a7032729c0f2101168d0c4ca86fb55a4cf6a36d31431be1c53a3bd7411bb24e8832410289fa6f3b";
  }
}

namespace cryptonote
{
  enum network_type : uint8_t
  {
    MAINNET = 0,
    TESTNET,
    STAGENET,
    FAKECHAIN,
    UNDEFINED = 255
  };
  struct config_t
  {
    uint64_t const CRYPTONOTE_PUBLIC_ADDRESS_BASE58_PREFIX;
    uint64_t const CRYPTONOTE_PUBLIC_INTEGRATED_ADDRESS_BASE58_PREFIX;
    uint64_t const CRYPTONOTE_PUBLIC_SUBADDRESS_BASE58_PREFIX;
    uint16_t const P2P_DEFAULT_PORT;
    uint16_t const RPC_DEFAULT_PORT;
    uint16_t const ZMQ_RPC_DEFAULT_PORT;
    boost::uuids::uuid const NETWORK_ID;
    std::string const GENESIS_TX;
  };
  inline const config_t& get_config(network_type nettype)
  {
    static const config_t mainnet = {
      ::config::CRYPTONOTE_PUBLIC_ADDRESS_BASE58_PREFIX,
      ::config::CRYPTONOTE_PUBLIC_INTEGRATED_ADDRESS_BASE58_PREFIX,
      ::config::CRYPTONOTE_PUBLIC_SUBADDRESS_BASE58_PREFIX,
      ::config::P2P_DEFAULT_PORT,
      ::config::RPC_DEFAULT_PORT,
      ::config::ZMQ_RPC_DEFAULT_PORT,
      ::config::NETWORK_ID,
      ::config::GENESIS_TX,
    };
    static const config_t testnet = {
      ::config::testnet::CRYPTONOTE_PUBLIC_ADDRESS_BASE58_PREFIX,
      ::config::testnet::CRYPTONOTE_PUBLIC_INTEGRATED_ADDRESS_BASE58_PREFIX,
      ::config::testnet::CRYPTONOTE_PUBLIC_SUBADDRESS_BASE58_PREFIX,
      ::config::testnet::P2P_DEFAULT_PORT,
      ::config::testnet::RPC_DEFAULT_PORT,
      ::config::testnet::ZMQ_RPC_DEFAULT_PORT,
      ::config::testnet::NETWORK_ID,
      ::config::testnet::GENESIS_TX,
    };
    static const config_t stagenet = {
      ::config::stagenet::CRYPTONOTE_PUBLIC_ADDRESS_BASE58_PREFIX,
      ::config::stagenet::CRYPTONOTE_PUBLIC_INTEGRATED_ADDRESS_BASE58_PREFIX,
      ::config::stagenet::CRYPTONOTE_PUBLIC_SUBADDRESS_BASE58_PREFIX,
      ::config::stagenet::P2P_DEFAULT_PORT,
      ::config::stagenet::RPC_DEFAULT_PORT,
      ::config::stagenet::ZMQ_RPC_DEFAULT_PORT,
      ::config::stagenet::NETWORK_ID,
      ::config::stagenet::GENESIS_TX,
    };
    switch (nettype)
    {
      case MAINNET: return mainnet;
      case TESTNET: return testnet;
      case STAGENET: return stagenet;
      case FAKECHAIN: return mainnet;
      default: throw std::runtime_error("Invalid network type");
    }
  }
}*/
