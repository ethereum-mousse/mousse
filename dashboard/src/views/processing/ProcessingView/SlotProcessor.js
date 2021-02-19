import React from 'react';
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

  const [slot, setSlot] = React.useState(0);
  const handleChangeSlot = (event) => {
    setSlot(event.target.value);
  };

  const [situation, setSituation] = React.useState('normal');
  const handleChangeSituation = (event) => {
    setSituation(event.target.value);
  };

  const handleSubmit = event => {
    event.preventDefault();

    console.log(slot);
    console.log(situation);

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

    let endpoint = "http://localhost:3030/simulator/slot/";
    endpoint += situation_to_endpoint[situation] + "/";
    endpoint += slot;
    console.log(endpoint);

    fetch(endpoint, {
      method: "POST",
    })
      .then(response => console.log(response))
      // .then(response => response.json())
      // .then(response => console.log("Success:", JSON.stringify(response)))
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
              <TextField
                id="slot"
                label="Slot"
                placeholder="0"
                margin="normal"
                onChange={handleChangeSlot}
                variant="outlined"
              />
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
        >
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
