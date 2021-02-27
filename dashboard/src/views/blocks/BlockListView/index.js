import React from 'react';
import {
  Box,
  Container,
  Grid,
  makeStyles,
} from '@material-ui/core';
import Page from 'src/components/Page';
import Results from './Results';
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

  return (
    <Page
      className={classes.root}
      title="Blocks"
    >
      <Grid>
        <Container maxWidth={false}>
          {/* <Toolbar /> */}
          <Box mt={3}>
            <CurrentSlotContext.Consumer>
              {value => (
                <Results current_slot={value.current_slot} />
              )}
            </CurrentSlotContext.Consumer>
          </Box>
        </Container>
      </Grid>
    </Page>
  );
};

export default BlockListView;
