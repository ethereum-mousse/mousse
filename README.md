![](mousse.png)

Mousse is an Ethereum 2.0 emulator for local testing of Eth2 applications (mainly Rollups).


## Getting Started

### Run Eth2 Emulator
You can run the Ethereum 2.0 emulator with the following commands:
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
![](dashboard.png)

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

#### Options
- `REACT_APP_PORT_NUMBER`: Port number of the eth2 emulator. Default: 3030.

## Eth2 Emulator Server
The implementation of the emulator server is in the `http_api` directory, and the Ethereum 2.0 simulator `simulator` is running inside. The emulator API definition is located in the [http_api/reference](http_api/reference) directory.

