#### [Gossip Glome.rs: Fly.io's Distributed Systems Challenges in Rust][dist-sys]

[![Tests][badge_gha_tests]][gha_tests]
[![Security Audit][badge_gha_audit]][gha_audit]
[![codecov][badge_codecov]][codecov]

_"We’ve teamed up with [Kyle Kingsbury][kyle], author of [Jepsen][jepsen], to build this series of [distributed systems challenges][dist-sys] so you can try your hand and see how your skills stack up._

_The challenges are built on top of a platform called [Maelstrom][maelstrom], which in turn, is built on Jepsen. This platform lets you build out a “node” in your distributed system and Maelstrom will handle the routing of messages between the those nodes. This lets Maelstrom inject failures and perform verification checks based on the consistency guarantees required by each challenge."_

_\- Fly\.io_

#### [Challenge 1: Echo][1] ✅ [![Echo][badge_gha_echo]][gha_echo]

- [x] Implement a simple echo service that receives a specially formatted "echo" message and payload and responds to each client with an "echo_ok" response with the matching payload.

#### [Challenge 2: Unique ID Generation][2] ✅ [![Unique IDs][badge_gha_unique]][gha_unique]

- [x] Implement a globally-unique ID generation system that is totally available (i.e It can continue to operate even in the face of network partitions).

#### [Challenge 3a: Single-Node Broadcast][3a] ✅ [![Broadcast 3a][badge_gha_broadcast-3a]][gha_broadcast-3a]

- [x] Implement a broadcast system that gossips messages to a single node.

#### [Challenge 3b: Multi-Node Broadcast][3b] ✅ [![Broadcast 3b][badge_gha_broadcast-3b]][gha_broadcast-3b]

- [x] Expand on the previous solution and gossip messages between multiple nodes in a cluster with no network partitions.

#### [Challenge 3c: Fault Tolerant Broadcast][3c] ✅ [![Broadcast 3c][badge_gha_broadcast-3c]][gha_broadcast-3c]

- [x] Improve the previous solution and gossip messages between multiple nodes even when there are network partitions.

#### [Challenge 3d: Efficient Broadcast, Part I][3d] ✅ [![Broadcast 3d][badge_gha_broadcast-3d]][gha_broadcast-3d]

- [x] Optimize the solution so it works well on a slow network with and without network partitions.
#### [Challenge 3e: Efficient Broadcast, Part II][3e] ✅ [![Broadcast 3e][badge_gha_broadcast-3e]][gha_broadcast-3e]

- [x] Optimize the solution so it meets the given performance metrics.

#### [Challenge 4: Grow-Only Counter][4] ✅ [![GCounter][badge_gha_gcounter]][gha_gcounter]
- [x] Implement a custom [sequentially-consistent][sequential] key/value store.
    - Note: The challenge said to use a provided key/value store service, but I made my own to better understand what's involved.
- [x] Implement a stateless grow-only counter and have all nodes use the custom KV store to handle adding a delta to the counter and reading the correct value.
- [x] Ensure the solution works across multiple nodes even when there are network partitions.

#### [Challenge 5: Kafka-Style Log][5a] 🚧📆

- [ ] Implement a replicated log service similar to [Kafka][kafka].
- [ ] Implement the requirements using a [linearizable][linearizability] key/value store.
- [ ] Ensure the solution works across multiple nodes.
- [ ] Increase the efficiency by evaluating bottlenecks, reduce the probability of CaS failures, and use more efficient [consistency models][consistency] for certain operations where appropriate.

#### [Challenge 6: Totally-Available Transactions][6a] 🚧📆
- [ ] Implement a key/value store that supports transactions and use it to perform all the operations within the transactions on a single node.
    - The goal is to support weak consistency while also being totally available.
- [ ] Implement [Totally-Available][consistency], [Read Uncommitted][read_uncommitted] Transactions across multiple nodes.
- [ ] Implement [Totally-Available][consistency], [Read Committed][read_committed] Transactions across multiple nodes.
- [ ] Ensure the solution works when there are network partitions.

#### 🔋 Code Coverage
[<img src="https://codecov.io/gh/jamrok/distributed-systems-rs/branch/main/graphs/sunburst.svg?token=W4IQDQ9VEX" width=150>][codecov]

[1]: https://fly.io/dist-sys/1
[2]: https://fly.io/dist-sys/2/
[3a]: https://fly.io/dist-sys/3a/
[3b]: https://fly.io/dist-sys/3b/
[3c]: https://fly.io/dist-sys/3c/
[3d]: https://fly.io/dist-sys/3d/
[3e]: https://fly.io/dist-sys/3e/
[4]: https://fly.io/dist-sys/4/
[5a]: https://fly.io/dist-sys/5a/
[5b]: https://fly.io/dist-sys/5b/
[5c]: https://fly.io/dist-sys/5c/
[6a]: https://fly.io/dist-sys/6a/
[6b]: https://fly.io/dist-sys/6b/
[6c]: https://fly.io/dist-sys/6c/
[badge_codecov]: https://codecov.io/gh/jamrok/distributed-systems-rs/branch/main/graph/badge.svg?token=W4IQDQ9VEX
[badge_gha_audit]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/audit.yml/badge.svg
[badge_gha_broadcast-3a]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-broadcast-3a.yml/badge.svg
[badge_gha_broadcast-3b]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-broadcast-3b.yml/badge.svg
[badge_gha_broadcast-3c]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-broadcast-3c.yml/badge.svg
[badge_gha_broadcast-3d]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-broadcast-3d.yml/badge.svg
[badge_gha_broadcast-3e]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-broadcast-3e.yml/badge.svg
[badge_gha_echo]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-echo.yml/badge.svg
[badge_gha_gcounter]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-gcounter.yml/badge.svg
[badge_gha_tests]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/tests.yml/badge.svg
[badge_gha_unique]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-unique-ids.yml/badge.svg
[codecov]: https://app.codecov.io/gh/jamrok/distributed-systems-rs
[consistency]: https://jepsen.io/consistency
[dist-sys]: https://fly.io/dist-sys
[gha_audit]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/audit.yml
[gha_broadcast-3a]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-broadcast-3a.yml
[gha_broadcast-3b]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-broadcast-3b.yml
[gha_broadcast-3c]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-broadcast-3c.yml
[gha_broadcast-3d]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-broadcast-3d.yml
[gha_broadcast-3e]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-broadcast-3e.yml
[gha_echo]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-echo.yml
[gha_gcounter]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-gcounter.yml
[gha_tests]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/tests.yml
[gha_unique]: https://github.com/jamrok/distributed-systems-rs/actions/workflows/challenge-unique-ids.yml
[git_hooks]: https://git-scm.com/docs/githooks
[jepsen]: https://jepsen.io
[kafka]: https://kafka.apache.org/
[kyle]: https://aphyr.com/about
[linearizability]: https://jepsen.io/consistency/models/linearizable
[maelstrom]: https://github.com/jepsen-io/maelstrom
[read_committed]: https://jepsen.io/consistency/models/read-committed
[read_uncommitted]: https://jepsen.io/consistency/models/read-uncommitted
[sequential]: https://jepsen.io/consistency/models/sequential
