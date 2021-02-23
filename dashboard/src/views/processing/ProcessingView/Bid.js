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
} from '@material-ui/core';
import InsertDriveFile from '@material-ui/icons/InsertDriveFile';
import bytesToHex from 'src/utils/bytesToHex';

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

  const [slot, setSlot] = useState(0);
  const handleChangeSlot = (event) => {
    setSlot(event.target.value);
  };

  const [point, setPoint] = useState(0);
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

  const toBase64 = file => new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);
    reader.onload = () => resolve(reader.result.replace(/data:.*\/.*;base64,/, ''));
    reader.onerror = error => reject(error);
  });

  const [file, setFile] = React.useState();
  const handleChangeFile = async (event) => {
    let file = await toBase64(event.target.files[0]);
    setFile(file);

    let endpoint = "http://localhost:3030/utils/data_commitment";
    let url = new URL(endpoint);
    let params = {
      data: file,
    };

    Object.keys(params).forEach(key => url.searchParams.append(key, params[key]));
    fetch(url, {
      method: "GET",
    })
      .then(response => response.json())
      .then(commitment => {
        setPoint(bytesToHex(commitment.point));
        setLength(commitment.length);
      })
      .catch(error => console.error("Error:", error));
  };

  const handleSubmit = event => {
    event.preventDefault();

    let body = JSON.stringify({
      shard: parseInt(shard),
      slot: parseInt(slot),
      commitment: {
        point: parseInt(point),
        length: parseInt(length),
      },
      fee: parseInt(fee)
    });

    let endpoint = "http://localhost:3030/data_market/bid";
    fetch(endpoint, {
      method: "POST",
      body: body,
      headers: {
        "Content-Type": "application/json"
      }
    })
      .then(response => console.log(response))
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
                  margin="normal"
                  padding="normal"
                  name="shard"
                  onChange={handleChangeShard}
                  variant="outlined"
                  placeholder="0"
                  value={shard}
                />
                <TextField
                  label="Slot"
                  margin="normal"
                  padding="normal"
                  name="slot"
                  onChange={handleChangeSlot}
                  variant="outlined"
                  placeholder="0"
                  value={slot}
                />
                <TextField
                  label="Fee (Gwei)"
                  margin="normal"
                  name="fee"
                  onChange={handleChangeFee}
                  variant="outlined"
                  placeholder="0"
                  value={fee}
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
                  label="Point"
                  margin="normal"
                  name="point"
                  onChange={handleChangePoint}
                  variant="outlined"
                  placeholder="0"
                  value={point}
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
                />
              </FormControl>

            </Grid>
            <Grid item xs={6} className={classes.upload}>
              <Box align="center">
                <input
                  accept="image/*"
                  className={classes.input}
                  id="contained-button-file"
                  // multiple
                  type="file"
                  onChange={handleChangeFile}
                />
                <label htmlFor="contained-button-file">
                  <Button variant="contained" color="primary" component="span">
                    SELECT FILE
                  </Button>
                </label>
                <input accept="image/*" className={classes.input} id="icon-button-file" type="file" />
                <label htmlFor="icon-button-file">
                  <IconButton color="primary" aria-label="upload picture" component="span">
                    <InsertDriveFile />
                  </IconButton>
                </label>
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
