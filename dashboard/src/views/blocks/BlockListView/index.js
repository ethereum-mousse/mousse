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

const BlockListView = () => {
  const classes = useStyles();

  const [blocks, setBlocks] = useState([]);

  const [count, setCount] = useState(10);
  const [page, setPage] = useState(0);

  const updateBlocks = (count, page) => {
    let endpoint = "http://localhost:" + process.env.REACT_APP_EMULATOR_PORT_NUMBER + "/beacon/blocks";
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
      .then(new_blocks => {
        if (blocks !== new_blocks) {
          new_blocks.reverse();
          setBlocks(new_blocks);
        }
      })
      .catch(error => console.error("Error:", error));
  }

  useEffect(() => {
    updateBlocks(count, page);
  }, []);

  return (
    <Page
      className={classes.root}
      title="Blocks"
    >
      <Grid>
        <Container maxWidth={false}>
          <Toolbar
            count={count}
            page={page}
            updateBlocks={updateBlocks}
          />
          <Box mt={3}>
            <CurrentSlotContext.Consumer>
              {value => (
                <Results
                  current_slot={value.current_slot}
                  count={count} setCount={setCount}
                  page={page} setPage={setPage}
                  blocks={blocks}
                  updateBlocks={updateBlocks} />
              )}
            </CurrentSlotContext.Consumer>
          </Box>
        </Container>
      </Grid>
    </Page>
  );
};

export default BlockListView;
