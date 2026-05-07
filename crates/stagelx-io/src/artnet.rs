// Phase 3: Art-Net Tx/Rx over UDP port 6454.
//
// Planned:
//   ArtNetNode — sends ArtDMX packets for each active universe
//   ArtNetListener — receives ArtDMX from external consoles, forwards to DmxEngine source
//   ArtPoll / ArtPollReply for node discovery
//
// Run in a dedicated tokio task; bridge to Bevy via crossbeam_channel.
