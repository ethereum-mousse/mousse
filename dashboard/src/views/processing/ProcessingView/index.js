import React, { useState, useEffect } from 'react';
import {
  Box,
  Container,
  makeStyles
} from '@material-ui/core';
import Page from 'src/components/Page';
import SlotProcessor from './SlotProcessor';
import Bid from './Bid';
import axios from 'axios';

const useStyles = makeStyles((theme) => ({
  root: {
    backgroundColor: theme.palette.background.dark,
    minHeight: '100%',
    paddingBottom: theme.spacing(3),
    paddingTop: theme.spacing(3)
  }
}));

const ProcessingView = () => {
  const classes = useStyles();

  const [head, setHead] = useState(null);
  const [current_slot, setCurrentSlot] = useState(null);

  useEffect(() => {
    const fetchData = async () => {
      const result = await axios(
        'http://localhost:3030/beacon/blocks/head'
      );

      setHead(result.data);
      setCurrentSlot(result.data.slot);
    };

    fetchData();
  }, []);

  return (
    <Page
      className={classes.root}
      title="Processing"
    >
      <Container maxWidth="md">
        <SlotProcessor current_slot={current_slot} setCurrentSlot={setCurrentSlot} />
        <Box mt={3}>
          <Bid />
        </Box>
      </Container>
    </Page>
  );
};

export default ProcessingView;
