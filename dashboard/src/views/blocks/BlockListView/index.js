import React, { useState, useEffect } from 'react';
import {
  Box,
  Container,
  Grid,
  makeStyles,
} from '@material-ui/core';
import Page from 'src/components/Page';
import Results from './Results';
// import Toolbar from './Toolbar';
import axios from 'axios';

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

  useEffect(() => {
    const fetchData = async () => {
      const result = await axios(
        'http://localhost:' + process.env.REACT_APP_PORT_NUMBER + '/beacon/blocks'
      );

      let blocks = result.data;
      setBlocks(blocks);
    };

    fetchData();
  }, []);

  return (
    <Page
      className={classes.root}
      title="Blocks"
    >
      <Grid>
        <Container maxWidth={false}>
          {/* <Toolbar /> */}
          <Box mt={3}>
            <Results blocks={blocks} />
          </Box>
        </Container>
      </Grid>
    </Page>
  );
};

export default BlockListView;
