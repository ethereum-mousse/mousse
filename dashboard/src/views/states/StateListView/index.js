import React, { useState, useEffect } from 'react';
import {
  Box,
  Container,
  Grid,
  makeStyles,
} from '@material-ui/core';
import Page from 'src/components/Page';
import Results from './Results';
import axios from 'axios';

const useStyles = makeStyles((theme) => ({
  root: {
    backgroundColor: theme.palette.background.dark,
    minHeight: '100%',
    paddingBottom: theme.spacing(3),
    paddingTop: theme.spacing(3)
  }
}));

const StateListView = () => {
  const classes = useStyles();
  const [states, setStates] = useState([]);

  useEffect(() => {
    const fetchData = async () => {
      const result = await axios(
        'http://localhost:' + process.env.REACT_APP_PORT_NUMBER + '/beacon/states'
      );

      let states = result.data;
      console.log(states);
      setStates(states);
    };

    fetchData();
  }, []);

  return (
    <Page
      className={classes.root}
      title="States"
    >
      <Grid>
        <Container maxWidth={false}>
          <Box mt={3}>
            <Results states={states} />
          </Box>
        </Container>
      </Grid>
    </Page>
  );
};

export default StateListView;
