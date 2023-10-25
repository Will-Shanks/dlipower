use clap::{Parser, Subcommand};
use dlipower::powerstrip::{PowerStrip, Status};
use scc_conf::Conf;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let conf = scc_conf::get_config().unwrap();

    match args.command {
        Commands::Status => {
            println!("{:?}", get_status(&conf).await);
        }
        Commands::Team(team) => {
            let team_conf = &conf.teams[team.team as usize];
            set_port(
                &conf.power_switch[team_conf.ps as usize],
                team_conf.ps_port,
                team.status,
            )
            .await;
        }
        Commands::Port(port) => {
            set_port(
                &conf.power_switch[port.switch as usize],
                port.port,
                port.status,
            )
            .await;
        }
    }
}

async fn get_status(conf: &Conf) -> Vec<Vec<Status>> {
    let mut resp = vec![];
    for switch in conf.power_switch.iter() {
        let ps = PowerStrip::new(
            switch.user.clone(),
            switch.password.clone(),
            switch.ip.clone(),
        )
        .await
        .unwrap();
        resp.push(ps.status().await.unwrap());
    }
    resp
}

async fn set_port(switch: &scc_conf::PowerStrip, port: u8, status: Status) {
    let ps = PowerStrip::new(
        switch.user.clone(),
        switch.password.clone(),
        switch.ip.clone(),
    )
    .await
    .unwrap();
    ps.update(port, status).await.unwrap();
}

#[derive(Debug, clap::Args)]
struct Team {
    team: u8,
    status: Status,
}
#[derive(Debug, clap::Args)]
struct Port {
    switch: u8,
    port: u8,
    status: Status,
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Status,
    Team(Team),
    Port(Port),
}
