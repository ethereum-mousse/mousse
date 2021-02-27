import React, { useState, useEffect } from 'react';
import {
  Box,
  Container,
  makeStyles
} from '@material-ui/core';
import Page from 'src/components/Page';
import SlotProcessor from './SlotProcessor';
import Bid from './Bid';
import SuccessDialog from './SuccessDialog';
import axios from 'axios';
import { CurrentSlotContext } from 'src/contexts/CurrentSlotContext';

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

  const [success_open, setSuccessOpen] = useState(false);

  // useEffect(() => {
  //   const fetchData = async () => {
  //     const result = await axios(
  //       'http://localhost:' + process.env.REACT_APP_PORT_NUMBER + '/beacon/blocks/head'
  //     );

  //     setHead(result.data);
  //     if (result.data) {
  //       setCurrentSlot(result.data.slot);
  //     }
  //   };

  //   fetchData();
  // }, []);

  return (
    <Page
      className={classes.root}
      title="Processing"
    >
      <CurrentSlotContext.Consumer>
        {value => (
          <Container maxWidth="md">
            <SlotProcessor current_slot={value.current_slot} setCurrentSlot={value.setCurrentSlot} setSuccessOpen={setSuccessOpen} />
            <Box mt={3}>
              <Bid current_slot={value.current_slot} setSuccessOpen={setSuccessOpen} />
            </Box>
          </Container>
        )}
      </CurrentSlotContext.Consumer>
      <SuccessDialog success_open={success_open} setSuccessOpen={setSuccessOpen} />
    </Page >
  );
};

export default ProcessingView;
