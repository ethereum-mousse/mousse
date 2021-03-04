import React, { useState } from 'react';
import {
  Container,
  makeStyles
} from '@material-ui/core';
import Page from 'src/components/Page';
import Server from './Server';
import SuccessDialog from 'src/components/SuccessDialog';

const useStyles = makeStyles((theme) => ({
  root: {
    backgroundColor: theme.palette.background.dark,
    minHeight: '100%',
    paddingBottom: theme.spacing(3),
    paddingTop: theme.spacing(3)
  }
}));

const SettingsView = () => {
  const classes = useStyles();

  const [success_open, setSuccessOpen] = useState(false);

  return (
    <Page
      className={classes.root}
      title="Settings"
    >
      <Container maxWidth="sm">
        <Server setSuccessOpen={setSuccessOpen} />
      </Container>
      <SuccessDialog success_open={success_open} setSuccessOpen={setSuccessOpen} />
    </Page>
  );
};

export default SettingsView;
