import React, { useState, useEffect } from 'react';
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
  makeStyles,
  FormControl,
  FormLabel,
  TextField,
  FormGroup,
  Switch,
  // FormHelperText
} from '@material-ui/core';
// import { result } from 'lodash';
import axios from 'axios';

const useStyles = makeStyles(({
  root: {},
  item: {
    display: 'flex',
    flexDirection: 'column'
  }
}));

const Server = ({ className, ...rest }) => {
  const classes = useStyles();

  const [state, setState] = useState({
    auto: false,
    slot_time: null,
    failure_rate: null,
  });

  useEffect(() => {
    const fetchData = async () => {
      const result = await axios(
        'http://localhost:' + process.env.REACT_APP_EMULATOR_PORT_NUMBER + '/config'
      );

      if (result.data) {
        setState(result.data);
      }
    };

    fetchData();
  }, []);

  const handleChange = (event) => {
    setState({ ...state, [event.target.name]: event.target.checked });
  };

  const handleChangeSlotTime = (event) => {
    setState({ ...state, "slot_time": parseInt(event.target.value) });
  };

  const handleChangeFailureRate = (event) => {
    setState({ ...state, "failure_rate": parseFloat(event.target.value) });
  };

  const handleSubmit = event => {
    event.preventDefault();
    let endpoint = "http://localhost:" + process.env.REACT_APP_EMULATOR_PORT_NUMBER + "/config";

    let body = JSON.stringify(state);

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
        }
        else {
          console.log("Error:", JSON.stringify(response));
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
          // subheader=""
          title="Settings"
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
              md={8}
              sm={8}
              xs={8}
            >

              <FormControl component="fieldset">
                <FormLabel component="legend">Simulator Mode</FormLabel>
                <FormGroup>
                  <FormControlLabel
                    control={<Switch checked={state.auto} onChange={handleChange} name="auto" />}
                    label="Auto Mode"
                  />
                </FormGroup>
                <TextField
                  label="Slot Time (Seconds)"
                  type="number"
                  inputProps={{ min: 0 }}
                  margin="normal"
                  padding="normal"
                  name="SlotTime"
                  onChange={handleChangeSlotTime}
                  variant="outlined"
                  disabled={!state.auto}
                  value={state.slot_time}
                  defaultValue={12}
                />
                <TextField
                  label="Failure Rate"
                  type="number"
                  inputProps={{ min: 0, max: 1, step: 0.1 }}
                  margin="normal"
                  padding="normal"
                  name="FailureRate"
                  onChange={handleChangeFailureRate}
                  variant="outlined"
                  disabled={!state.auto}
                  value={state.failure_rate}
                  defaultValue={0}
                />
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
            Save
          </Button>
        </Box>
      </Card>
    </form>
  );
};

Server.propTypes = {
  className: PropTypes.string
};

export default Server;
