import React, { useState } from 'react';
import PropTypes from 'prop-types';
import clsx from 'clsx';
import {
  Box,
  Button,
  Card,
  CardContent,
  CardHeader,
  Divider,
  FormControlLabel,
  Grid,
  TextField,
  makeStyles,
  Radio,
  FormControl,
  FormLabel,
  RadioGroup,
} from '@material-ui/core';
// import { green } from '@material-ui/core/colors';

const useStyles = makeStyles(({
  root: {},
  item: {
    display: 'flex',
    flexDirection: 'column',
    '& > button': {
      margin: '5px'
    },
  },
  vcenter: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
  },
  submit_buttons: {
    '& > button': {
      margin: '5px'
    },
  }
  // green_radio: {
  //   color: green[400],
  //   '&$checked': {
  //     color: green[600],
  //   },
  //   checked: {},
  // },
}));


const SlotProcessor = ({ className, ...rest }) => {
  const classes = useStyles();

  const [slot, setSlot] = useState(rest.current_slot);
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

  const [situation, setSituation] = useState('normal');
  const handleChangeSituation = (event) => {
    setSituation(event.target.value);
  };

  const handleSubmit = event => {
    event.preventDefault();

    const situation_to_endpoint = {
      "normal": "process",
      "without_shard_data_inclusion": "process_without_shard_data_inclusion",
      "without_shard_blob_proposal": "process_without_shard_blob_proposal",
      "without_shard_header_inclusion": "process_without_shard_header_inclusion",
      "without_shard_header_confirmation": "process_without_shard_header_confirmation",
      "without_beacon_chain_finality": "process_without_beacon_chain_finality",
      "without_beacon_block_proposal": "process_without_beacon_block_proposal",
      "random": "process_random"
    }

    let endpoint = "http://localhost:" + process.env.REACT_APP_PORT_NUMBER + "/simulator/slot/";
    endpoint += situation_to_endpoint[situation] + "/";
    endpoint += slot;

    fetch(endpoint, {
      method: "POST",
    })
      .then(response => {
        if (response.status === 200) {
          console.log("Success");
          rest.setCurrentSlot(slot);
          rest.setSuccessOpen(true);
        }
        else {
          response.json().then(() => {
            console.log("Error:", JSON.stringify(response));
          })
        }
      })
      .catch(error => console.error("Error:", error));
  };

  const handleClickInitSimulator = event => {
    event.preventDefault();

    let endpoint = "http://localhost:" + process.env.REACT_APP_PORT_NUMBER + "/simulator/init";

    fetch(endpoint, {
      method: "POST",
    })
      .then(response => {
        if (response.status === 200) {
          console.log("Success");
          rest.setSuccessOpen(true);
        }
        else {
          response.json().then(() => {
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
          title="Slot"
          subheader="Process slots"
        />
        <Divider />
        <CardContent>
          <Grid
            container
            spacing={6}
            wrap="wrap"
          >
            <Grid
              className={classes.item}
              item
              md={6}
              sm={6}
              xs={6}
            >
              {slot_invalid ?
                <TextField
                  error
                  id="slot"
                  type="number"
                  label="Slot"
                  placeholder="0"
                  margin="normal"
                  onChange={handleChangeSlot}
                  variant="outlined"
                  helperText="Invalid slot."
                  value={slot}
                />
                :
                <TextField
                  id="slot"
                  type="number"
                  label="Slot"
                  placeholder="0"
                  margin="normal"
                  onChange={handleChangeSlot}
                  variant="outlined"
                  value={slot}
                />
              }
            </Grid>
            <Grid
              className={classes.item}
              item
              md={6}
              sm={6}
              xs={6}
            >
              <FormControl
                component="fieldset"
                margin="normal"
              >
                <FormLabel component="legend">Situation</FormLabel>
                <RadioGroup aria-label="situation" name="situation" value={situation} onChange={handleChangeSituation}>
                  <FormControlLabel value="normal" control={<Radio />} label="normal" />
                  <FormControlLabel value="without_shard_data_inclusion" control={<Radio />} label="without shard data inclusion" />
                  <FormControlLabel value="without_shard_blob_proposal" control={<Radio />} label="without shard blob proposal" />
                  <FormControlLabel value="without_shard_header_inclusion" control={<Radio />} label="without shard header inclusion" />
                  <FormControlLabel value="without_shard_header_confirmation" control={<Radio />} label="without shard header confirmation" />
                  <FormControlLabel value="without_beacon_chain_finality" control={<Radio />} label="without beacon chain finality" />
                  <FormControlLabel value="without_beacon_block_proposal" control={<Radio />} label="without beacon block proposal" />
                  <FormControlLabel value="random" control={<Radio />} label="random" />
                </RadioGroup>
              </FormControl>
            </Grid>

          </Grid>
        </CardContent>
        <Divider />
        <Box
          display="flex"
          justifyContent="flex-end"
          p={2}
          className={classes.submit_buttons}
        >
          <Button
            variant="contained"
            onClick={handleClickInitSimulator}
          >
            Init Simulator
          </Button>
          <Button
            color="primary"
            variant="contained"
            type="submit"
          >
            Process
          </Button>
        </Box>
      </Card>
    </form>
  );
};

SlotProcessor.propTypes = {
  className: PropTypes.string
};

export default SlotProcessor;
