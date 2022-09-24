# Rusty Cannon

This repo contains a Proof-of-Concept that demonstrates it is possible to have Rust based 
state transition functions (STF) using [Cannon]. This project was started by [ec2] and [pepyakin] on
the [EthBerlin3] hackathon, but sadly we didn't have enough time to finish it in time.

## Arbitrary 

The STF that we implement here is called `arbitrary` for whatever reason. 

The STF itself is very trivial. The state is basically a mapping from 32 bytes account addresses to
balances represented as `u64`. The transactions are simple transfers of funds from one account to
another. The transactions do not have any signatures, so anyone can send any transaction. 

The blocks are also very simple. There is no separation between a header and a block, so they are
passed verbatim. Therefore, there is no need to commiting to the transaction root. Blocks do not 
have any receipts either. They are just:

```rust
struct Block {
    number: u64,
    parent: H256,
    state_root: H256,
    txns: Vec<Txn>,
}
```

One interesting distinction from the vanilla Cannon approach, is that the MIPS STF does not 
explicitly target Linux. Instead, we target bare-metal MIPS. By using the bare-metal target we can
tightly control interactions with the host (onchain verifier or offchain prover), what instructions
are emitted[^1]. Only one syscall is actually necessary to interact with system. I think this is
much better approach than be at mercy of the Go compiler and runtime (although probably not much 
could be done if geth to be used). The price is the lack of standard library, although that should 
be fixable.

[^1]: Turns out the way LLVM emits code is a bit more cunning. Unlike Go, it relies on .got table and uses some nasty stuff like `rdwhr` for getting the TLS address.

<pre>
<a href="./arbitrary">arbitrary</a>
├── <a href="./arbitrary/arbitrary-state-machine">arbitrary-state-machine</a>: The platform-agnostic implementation of a state transition function.
├── <a href="./arbitrary/arbitrary-prepare-mock">arbitrary-prepare-mock</a>: A program that creates a mock blockchain.
├── <a href="./arbitrary/arbitrary-prover-main">arbitrary-prover-main</a>: The MIPS STF that is used for proving.
│   ├── <a href="./arbitrary/arbitrary-prover-main/mips-unknown-none.json">mips-unknown-none.json</a>: The bare-metal target defintion for rustc/llvm.
</pre>

For implementation of the trie and other essentials, Wei's libraries were used since they were the 
most lightweight and easy to use. The problem with them they are a bit outdated and most importantly
the trie crate did not support compilation for `no_std`. The easiest way to fix this was to fork 
them and rip out all the unnecessary stuff. Here are those:

<pre>
├── <a href="./arbitrary/ethereum-bigint">ethereum-bigint</a>
├── <a href="./arbitrary/ethereum-hexutil">ethereum-hexutil</a>
├── <a href="./arbitrary/ethereum-rlp">ethereum-rlp</a>
├── <a href="./arbitrary/ethereum-trie">ethereum-trie</a>
</pre>

## How to run

Usage:

    $ git submodule update --init --recursive
    $ docker build . -t arbitrary-cannon/demo
    $ docker run -it --rm --name dev -v $(pwd):/code arbitrary-cannon/demo bash

After you got into the shell, run:

    $ ./demo/challenge_simple.sh
    $ ./demo/challenge_fault.sh

[ec2]: https://github.com/ec2
[pepyakin]: https://github.com/pepyakin
[arbitrary]: ./arbitrary/
[Cannon]: https://github.com/ethereum-optimism/cannon
[EthBerlin3]: https://ethberlin.ooo/
