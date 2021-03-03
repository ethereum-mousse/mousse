import React, { useState, useEffect } from 'react';
import {
  Box,
  Container,
  makeStyles
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

const LogListView = () => {
  const classes = useStyles();
  const [logs, setLogs] = useState([]);

  useEffect(() => {
    const fetchData = async () => {
      const result = await axios(
        'http://localhost:' + process.env.REACT_APP_EMULATOR_PORT_NUMBER + '/utils/request_logs'
      );

      let logs = result.data;
      logs.sort((a, b) => b.log_id - a.log_id);
      setLogs(logs);
    };

    fetchData();
  }, []);

  return (
    <Page
      className={classes.root}
      title="Logs"
    >
      <Container maxWidth="lg">
        <Box mt={3}>
          <Results logs={logs} />
        </Box>
      </Container>
    </Page>
  );
};

export default LogListView;
