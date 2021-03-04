![](mousse.png)

Mousse is an Ethereum 2.0 emulator for local testing of Eth2 applications (mainly Rollups).

## HTTP Server

The REST API definition can be found in the [http_api/reference](http_api/reference) directory.

Inside the server, the Eth2 data sharding simulator ([eth2_simulator](simulator/)) is running.

### Run Server
You can run the Ethereum 2.0 emulator server with the following commands:
```
$ cargo build --release
$ ./target/release/http_api
```

#### Flags
```
-a, --auto       Running simulator in auto mode. Default: false.
-h, --help       Prints help information
-V, --version    Prints version information
```
#### Options
```
-f, --failure-rate <FAILURE_RATE>    Failure rate for the auto mode. Default: 0.
-p, --port <PORT>                    Port number to listen on. Default: 3030.
-s, --slot-time <SLOT_TIME>          Slot time in seconds for the auto mode. Default: 12.
```

#### Logging
```
$ RUST_LOG=trace ./target/release/http_api 
```

## Dashboard
![dashboard](https://user-images.githubusercontent.com/20497787/109783408-511b4600-7c4d-11eb-8f58-634003d7a9c7.png)

### Run Dashboard
If you want to use the GUI to visualize and control the emulator, run the dashboard by entering the following commands.

Install:
```
$ cd dashboard
$ npm install
```

Run:
```
$ npm start
```

The default port of the emulator server that the dashboard connects to is 3030.
If you want to change the port, run the following command to start:
```
$ REACT_APP_EMULATOR_PORT_NUMBER=<PORT_NUMBER> npm start
```
