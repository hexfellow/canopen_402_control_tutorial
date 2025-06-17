use clap::Parser;
use futures_util::StreamExt;
use lazy_static::lazy_static;
use log::info;
use socketcan::tokio::CanFdSocket;
use socketcan::CanFilter;
use socketcan::EmbeddedFrame;
use socketcan::Frame;
use socketcan::SocketOptions;

lazy_static! {
    static ref MOTOR_POSITIONS: [tokio::sync::Mutex<Option<i32>>; 1] =
        [tokio::sync::Mutex::new(None)];
}

// Confidential File. IT IS FORBIDDEN to use our rust canopen lib in public demo projects in any form.
// In case some demo requires SDO read/write, directly replace all sdo r/w with direct [u8;8] data.
// mod canopen;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// Can interface name, e.g. can0
    #[arg(long, short)]
    can_interface: String,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = Args::parse();

    for (i, _) in MOTOR_POSITIONS.iter().enumerate() {
        let canbus = args.can_interface.clone();
        tokio::spawn(async move {
            let (_, mut rx) = {
                let canbus = CanFdSocket::open(canbus.as_str()).unwrap();
                let filter = CanFilter::new(0x381u32 + i as u32, 0x7FF);
                info!("Filter: {:?}", filter);
                canbus.set_filters(&[filter]).unwrap();
                canbus.split()
            };
            loop {
                let f = rx.next().await.unwrap().unwrap();
                {
                    if f.len() != 6 {
                        continue;
                    }
                    let pos =
                        i32::from_le_bytes([f.data()[2], f.data()[3], f.data()[4], f.data()[5]]);
                    *MOTOR_POSITIONS[i as usize].lock().await = Some(pos);
                }
            }
        });
    }
    loop {
        // Print out all motor positions
        let mut motor_positions: Vec<Option<i32>> = vec![];
        for (i, _) in MOTOR_POSITIONS.iter().enumerate() {
            let pos = MOTOR_POSITIONS[i].lock().await.clone();
            motor_positions.push(pos);
        }
        let mut log_string = "".to_string();
        for (i, pos) in motor_positions.iter().enumerate() {
            log_string += &format!("Motor{} position: {:?};", i, pos);
        }
        println!("{}", log_string);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
