const fs = require("fs")
const { basedir, deployed, getTrieNodesForCall } = require("../scripts/lib")

async function main() {
  let [c, m, mm] = await deployed()

  const blockNumberN = parseInt(process.env.BLOCK)
  // The challenged block contents to submit during the challenge. Will be checked on-chain and must
  // match the previously submitted (see preload.js) block hash.
  const REAL_BLOCK = process.env.REAL_BLOCK

  if (isNaN(blockNumberN) || REAL_BLOCK == undefined) {
    throw "usage: BLOCK=<number> REAL_BLOCK=<path> npx hardhat run challenge.js"
  }
  console.log("challenging block number", blockNumberN)
  console.log("real block is", REAL_BLOCK)
  const blockNp1 = fs.readFileSync(REAL_BLOCK)

  console.log(c.address, m.address, mm.address)

  // TODO: move this to lib, it's shared with the test
  let startTrie = JSON.parse(fs.readFileSync(basedir+"/golden.json"))

  const assertionRootBinary = fs.readFileSync(basedir+"/0_"+blockNumberN.toString()+"/output")
  var assertionRoot = "0x"
  for (var i=0; i<32; i++) {
    hex = assertionRootBinary[i].toString(16);
    assertionRoot += ("0"+hex).slice(-2);
  }
  console.log("asserting root", assertionRoot)
  let finalTrie = JSON.parse(fs.readFileSync(basedir+"/0_"+blockNumberN.toString()+"/checkpoint_final.json"))

  let preimages = Object.assign({}, startTrie['preimages'], finalTrie['preimages']);
  const finalSystemState = finalTrie['root']

  let args = [blockNumberN, blockNp1, assertionRoot, finalSystemState, finalTrie['step']]
  let cdat = c.interface.encodeFunctionData("initiateChallenge", args)
  let nodes = await getTrieNodesForCall(c, c.address, cdat, preimages)

  // run "on chain"
  for (n of nodes) {
    await mm.AddTrieNode(n)
  }
// TODO: Setting the gas limit explicitly here shouldn't be necessary, for some
//    weird reason (to be investigated), it is for L2.
//  let ret = await c.initiateChallenge(...args)
  let ret = await c.initiateChallenge(...args, { gasLimit: 10_000_000 })
  let receipt = await ret.wait()
  // ChallengeCreated event
  let challengeId = receipt.events[0].args['challengeId'].toNumber()
  let stateState = receipt.events[0].args['startState']
  let inputHash = receipt.events[0].args['inputHash']
  console.log("new challenge with id", challengeId, "and start state", stateState, "and input hash", inputHash)
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
