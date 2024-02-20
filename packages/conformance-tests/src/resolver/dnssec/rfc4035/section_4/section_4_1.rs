use dns_test::client::{Client, DigSettings};
use dns_test::name_server::NameServer;
use dns_test::record::RecordType;
use dns_test::tshark::{Capture, Direction};
use dns_test::zone_file::Root;
use dns_test::{Network, Resolver, Result, TrustAnchor, FQDN};

#[test]
#[ignore]
fn edns_support() -> Result<()> {
    let network = &Network::new()?;
    let ns = NameServer::new(dns_test::peer(), FQDN::ROOT, network)?.start()?;
    let resolver = Resolver::start(
        dns_test::subject(),
        &[Root::new(ns.fqdn().clone(), ns.ipv4_addr())],
        &TrustAnchor::empty(),
        network,
    )?;

    let mut tshark = resolver.eavesdrop()?;

    let client = Client::new(network)?;
    let settings = *DigSettings::default().authentic_data().recurse();
    let ans = client.dig(settings, resolver.ipv4_addr(), RecordType::SOA, &FQDN::ROOT)?;
    assert!(ans.status.is_servfail());

    tshark.wait_for_capture()?;

    let captures = tshark.terminate()?;

    let ns_addr = ns.ipv4_addr();
    for Capture { message, direction } in captures {
        if let Direction::Outgoing { destination } = direction {
            if destination == client.ipv4_addr() {
                continue;
            }

            // sanity check
            assert_eq!(ns_addr, destination);

            if destination == ns_addr {
                assert_eq!(Some(true), message.is_do_bit_set());
                assert!(message.udp_payload_size().unwrap() >= 1220);
            }
        }
    }

    Ok(())
}
