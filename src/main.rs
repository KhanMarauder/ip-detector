use std::sync::Arc;
use std::error::Error;
use surge_ping::{Client, ConfigBuilder, PingIdentifier, PingSequence};
use std::net::{IpAddr, Ipv4Addr};
use tokio::task::JoinSet;




#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	// ---- Create host discovery tables ----
	// Get arguments
	let args: Vec<String> = std::env::args()
		.map(|x| x.to_string())
		.filter(|x| !x.is_empty())
		.collect();

	// Check if proper arguments are specified
	if args.len() != 2 && args.len() != 3 {
		invalid_args_err();
		return Err("invalid arguments".into()); // Return error
	}

	// Create anotation table
	let mut octet0: Option<u8> = None;
	let mut octet1: Option<u8> = None;
	let mut octet2: Option<u8> = None;
	let mut octet3: Option<u8> = None;
	let timeout = args.get(3).unwrap_or(&String::new()).parse().unwrap_or(3);

	let host_ip_notation: Vec<String> = args[1]
		.split(".")
		.map(|x| x.to_string())
		.filter(|x| !x.is_empty())
		.collect();

	if host_ip_notation.len() != 4 {
		invalid_ip_notation();
		return Err("invalid syntax".into()); // Return error
	}

	// Convert into octets
	if let Ok(octet) = host_ip_notation[0].parse() {
		octet0 = Some(octet);
	} else if host_ip_notation[0] != "-" {
		invalid_ip_notation();
		return Err("invalid syntax".into()); // Return error
	};

	if let Ok(octet) = host_ip_notation[1].parse() {
		octet1 = Some(octet);
	} else if host_ip_notation[1] != "-" {
		invalid_ip_notation();
		return Err("invalid syntax".into()); // Return error
	}

	if let Ok(octet) = host_ip_notation[2].parse() {
		octet2 = Some(octet);
	} else if host_ip_notation[2] != "-" {
		invalid_ip_notation();
		return Err("invalid syntax".into()); // return error
	}

	if let Ok(octet) = host_ip_notation[3].parse() {
		octet3 = Some(octet);
	} else if host_ip_notation[3] != "-" {
		invalid_ip_notation();
		return Err("invalid syntax".into()); // Return error
	}


	// ---- Create pinger ----
	let config = ConfigBuilder::default().build(); // IPv4 support only
	let client = Arc::new(Client::new(&config)?);


	// ---- Send pings ----
	// Create flat list of targets
	let mut targets: Vec<Ipv4Addr> = vec![];
	for o0 in if let Some(x) = octet0 {x..=x} else {0..=254} {
		for o1 in if let Some(x) = octet1 {x..=x} else {0..=254} {
			for o2 in if let Some(x) = octet2 {x..=x} else {0..=254} {
				for o3 in if let Some(x) = octet3 {x..=x} else {0..=254} {
					targets.push(Ipv4Addr::from_octets([o0, o1, o2, o3]));
				}
			}
		}
	}

	// Send each target a ping
	let payload = [0; 56]; // Empty payload
	let mut pings = JoinSet::new();
	for (id, ip) in targets.iter().enumerate() {
		let client = Arc::clone(&client);
		let ip: IpAddr = ip.clone().into();
		pings.spawn(async move {
			let mut pinger = client.pinger(ip, PingIdentifier(id.clone() as u16)).await;
			pinger.timeout(std::time::Duration::from_secs(timeout));
			let result = pinger.ping(PingSequence(0), &payload).await;

			(ip, result)
		});
	}


	// ---- Wait for and read results from pings ----
	while let Some(ping_result) = pings.join_next().await {
		match ping_result {
			Ok((ip, Ok((_, duration)))) => println!("Host is up {ip} ({})", duration.as_secs_f32()),
			_ => {},
		}
	}
	

	// ---- Finished ----
	Ok(()) // Success
}



fn invalid_args_err() {
	println!("\x1b[91;1minvalid usage\x1b[0;1m:\x1b[0m expected usage:");
	println!("                             ip-detector \x1b[1m<host-ip-notation> [duration (default 3 secs)]\x1b[0m");
}

fn invalid_ip_notation() {
	println!("\x1b[91;1minvalid syntax\x1b[0;1m:\x1b[0m expected syntax:");
	println!(" \x1b[94;1m-->\x1b[0m \x1b[1mex.\x1b[0m ip-detector 10.0.1.-");
	println!("  \x1b[94;1m|\x1b[0m                  \x1b[93;1m~~~~~~ ~\x1b[0m");
	println!("  \x1b[94;1m|\x1b[0m                     \x1b[93:1m|   |\x1b[0m");
	println!("  \x1b[94;1m|\x1b[0m                     \x1b[93;1m:\x1b[0;1m shows the base, unchanging IP\x1b[0m");
	println!("  \x1b[94;1m|\x1b[0m                         \x1b[93;1m|\x1b[0m");
	println!("  \x1b[94;1m|\x1b[0m                         \x1b[93;1m:\x1b[0;1m shows the octet(s) that will be mutated 0-255 for host detection\x1b[0m");
}
