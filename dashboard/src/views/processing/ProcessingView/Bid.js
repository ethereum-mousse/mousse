import React, { useState } from 'react';
import PropTypes from 'prop-types';
import clsx from 'clsx';
import {
  Box,
  Button,
  Grid,
  Card,
  CardContent,
  CardHeader,
  Divider,
  TextField,
  makeStyles,
  IconButton,
  FormControl,
  FormLabel,
  FormHelperText,
} from '@material-ui/core';
import InsertDriveFile from '@material-ui/icons/InsertDriveFile';
import bytesToHex from 'src/utils/bytesToHex';
import hexToBytes from 'src/utils/hexToBytes';

const useStyles = makeStyles((theme) => ({
  root: {},
  input: {
    display: 'none',
  },
  upload: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
  },
  block_info: {
    flexDirection: 'row',
    '& > *': {
      margin: theme.spacing(1),
    }
  },
  commitment: {
    // flexDirection: 'column',
    width: '100%',
    '& > *': {
      margin: theme.spacing(1),
    }
  }
}));

const Bid = ({ className, ...rest }) => {
  const classes = useStyles();

  const [shard, setShard] = useState(0);
  const handleChangeShard = (event) => {
    setShard(event.target.value);
  };

  const [slot, setSlot] = useState(rest.current_slot + 1);
  const [slot_invalid, setSlotInvalid] = useState(false);
  const handleChangeSlot = (event) => {
    setSlot(event.target.value);
    if (rest.current_slot === null || event.target.value > rest.current_slot) {
      setSlotInvalid(false);
    }
    else {
      setSlotInvalid(true);
    }
  };

  const [point, setPoint] = useState("0x0");
  const handleChangePoint = (event) => {
    setPoint(event.target.value);
  };

  const [length, setLength] = useState(0);
  const handleChangeLength = (event) => {
    setLength(event.target.value);
  };

  const [fee, setFee] = useState(0);
  const handleChangeFee = (event) => {
    setFee(event.target.value);
  };

  const toBase64 = async file => {
    let array_buffer = await file.arrayBuffer();
    // The following code is caught in the stack limit:
    //   let base64_string = btoa(String.fromCharCode(...new Uint8Array(array_buffer)));
    // We need to use the for statement:
    let uintArray = new Uint8Array(array_buffer);
    let converted = "";
    uintArray.forEach(byte => {
      converted += String.fromCharCode(byte);
    });
    let base64_string = btoa(converted);
    return base64_string;
  };

  const [filename, setFilename] = useState("");

  const [encoded_file, setEncodedFile] = useState();
  const handleChangeFile = async (event) => {
    if (event.target.files.length === 0) {
      return;
    }
    setFilename(event.target.files[0].name);
    let encoded_file = await toBase64(event.target.files[0]);
    setEncodedFile(encoded_file);

    let endpoint = "http://localhost:" + process.env.REACT_APP_EMULATOR_PORT_NUMBER + "/utils/data_commitment";
    let body = JSON.stringify({
      data: encoded_file
    });

    fetch(endpoint, {
      method: "POST",
      body: body,
      headers: {
        "Content-Type": "application/json"
      }
    })
      .then(response => {
        if (response.status === 200) {
          console.log("Success");
          response.json().then(commitment => {
            setPoint("0x" + bytesToHex(commitment.point));
            setLength(commitment.length);
          })
        }
        else {
          console.log("Error:", JSON.stringify(response));
        }
      })
      .catch(error => console.error("Error:", error));
  };

  const handleSubmit = event => {
    event.preventDefault();

    let point_raw = hexToBytes(point.slice(2));

    let endpoint = 'http://localhost:' + process.env.REACT_APP_EMULATOR_PORT_NUMBER + '/shards/' + shard + '/bid_with_data';
    let body = JSON.stringify({
      bid: {
        shard: parseInt(shard),
        slot: parseInt(slot),
        commitment: {
          point: hexToBytes(point_raw),
          length: parseInt(length),
        },
        fee: parseInt(fee)
      },
      data: encoded_file,
    });

    fetch(endpoint, {
      method: "POST",
      body: body,
      headers: {
        "Content-Type": "application/json"
      }
    })
      .then(response => {
        if (response.status === 200) {
          console.log("Success");
          rest.setSuccessOpen(true);
        }
        else {
          response.json().then(response => {
            console.log("Error:", JSON.stringify(response));
          })
        }
      })
      .catch(error => console.error("Error:", error));
  };

  return (
    <form
      className={clsx(classes.root, className)}
      onSubmit={handleSubmit}
      {...rest}
    >
      <Card>
        <CardHeader
          title="Bid"
          subheader="Submit a bid"
        />
        <Divider />
        <CardContent>
          <Grid
            container
            wrap="wrap"
          >
            <Grid item xs={6}>
              <FormControl
                className={classes.block_info}
                component="fieldset"
                margin="normal"
              >
                <FormLabel component="legend">Block Info</FormLabel>
                <TextField
                  label="Shard"
                  type="number"
                  inputProps={{ min: 0 }}
                  margin="normal"
                  padding="normal"
                  name="shard"
                  onChange={handleChangeShard}
                  variant="outlined"
                  defaultValue={shard}
                />
                {slot_invalid ?
                  <TextField
                    label="Slot"
                    type="number"
                    inputProps={{ min: 0 }}
                    margin="normal"
                    padding="normal"
                    name="slot"
                    onChange={handleChangeSlot}
                    variant="outlined"
                    defaultValue={rest.current_slot + 1}
                    error
                    helperText="Invalid slot."
                  />
                  :
                  <TextField
                    label="Slot"
                    type="number"
                    inputProps={{ min: 0 }}
                    margin="normal"
                    padding="normal"
                    name="slot"
                    onChange={handleChangeSlot}
                    variant="outlined"
                    defaultValue={rest.current_slot + 1}
                  />
                }
                <TextField
                  label="Fee (Gwei)"
                  margin="normal"
                  name="fee"
                  onChange={handleChangeFee}
                  variant="outlined"
                  defaultValue={fee}
                />
              </FormControl>

              <FormControl
                className={classes.commitment}
                component="fieldset"
                margin="normal"
              >
                <FormLabel component="legend">Commitment</FormLabel>
                <TextField
                  fullWidth
                  label="Point (hex)"
                  margin="normal"
                  name="point"
                  onChange={handleChangePoint}
                  variant="outlined"
                  placeholder="0x0"
                  value={point}
                  disabled
                  helperText="Automatically calculated from a file."
                />
                <TextField
                  fullWidth
                  label="Length"
                  margin="normal"
                  name="length"
                  onChange={handleChangeLength}
                  variant="outlined"
                  placeholder="0"
                  value={length}
                  disabled
                  helperText="Automatically calculated from a file."
                />
              </FormControl>

            </Grid>
            <Grid item xs={6} className={classes.upload}>
              <Box align="center">
                <input
                  className={classes.input}
                  id="contained-button-file"
                  type="file"
                  onChange={handleChangeFile}
                />
                <label htmlFor="contained-button-file">
                  <Button variant="contained" color="primary" component="span">
                    SELECT FILE
                  </Button>
                </label>
                <input
                  className={classes.input}
                  id="icon-button-file"
                  type="file"
                  onChange={handleChangeFile}
                />
                <label htmlFor="icon-button-file">
                  <IconButton color="primary" aria-label="upload picture" component="span">
                    <InsertDriveFile />
                  </IconButton>
                </label>
                <FormHelperText>{filename}</FormHelperText>
              </Box>
            </Grid>
          </Grid>
        </CardContent>
        <Divider />
        <Box
          display="flex"
          justifyContent="flex-end"
          p={2}
        >
          <Button
            color="primary"
            variant="contained"
            type="submit"
          >
            SUBMIT
          </Button>
        </Box>
      </Card>
    </form >
  );
};

Bid.propTypes = {
  className: PropTypes.string
};

export default Bid;
