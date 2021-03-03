import React, { useState, useEffect } from 'react';
import {
  Box,
  Container,
  Grid,
  makeStyles,
} from '@material-ui/core';
import Page from 'src/components/Page';
import Results from './Results';
import Toolbar from './Toolbar';
import { CurrentSlotContext } from 'src/contexts/CurrentSlotContext';

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

  const [count, setCount] = useState(10);
  const [page, setPage] = useState(0);

  const updateStates = (count, page) => {
    let endpoint = "http://localhost:" + process.env.REACT_APP_PORT_NUMBER + "/beacon/states";
    let url = new URL(endpoint);
    let params = {
      count: count,
      page: page,
    };

    Object.keys(params).forEach(key => url.searchParams.append(key, params[key]));
    fetch(url, {
      method: "GET",
    })
      .then(response => response.json())
      .then(new_states => {
        if (states != new_states) {
          new_states.reverse();
          setStates(new_states);
        }
      })
      .catch(error => console.error("Error:", error));
  }

  useEffect(() => {
    updateStates(count, page);
  }, []);

  return (
    <Page
      className={classes.root}
      title="States"
    >
      <Grid>
        <Container maxWidth={false}>
          <Toolbar
            count={count}
            page={page}
            updateStates={updateStates}
          />
          <Box mt={3}>
            <CurrentSlotContext.Consumer>
              {value => (
                <Results
                  current_slot={value.current_slot}
                  count={count} setCount={setCount}
                  page={page} setPage={setPage}
                  states={states}
                  updateStates={updateStates} />
              )}
            </CurrentSlotContext.Consumer>
          </Box>
        </Container>
      </Grid>
    </Page>
  );
};

export default StateListView;
