const fs = require("fs")
const { basedir, deployed, getTrieNodesForCall } = require("../scripts/lib")
const { keccak256 } = require("ethers/lib/utils")

async function main() {
    let [c, _m, _mm] = await deployed()
    
    // Fill in the demo blockdata.
    for (i of [0, 1, 2, 3]) {
      let blockData = fs.readFileSync(basedir+"/0_"+i.toString()+"/block")
      let blockHash = keccak256(blockData)
      console.log("block", i, "blockHash", blockHash)
      // The challenge data for the block N, actually stores the block N+1.
      await c.addBlock(i + 1, blockHash)
    }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
