import React from 'react';
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
  Typography,
  makeStyles,
  FormControl,
  // FormLabel,
  TextField,
  FormGroup,
  Switch,
  // FormHelperText
} from '@material-ui/core';

const useStyles = makeStyles(({
  root: {},
  item: {
    display: 'flex',
    flexDirection: 'column'
  }
}));

const Server = ({ className, ...rest }) => {
  const classes = useStyles();

  const [state, setState] = React.useState({
    auto_mining: false,
  });

  const handleChange = (event) => {
    setState({ ...state, [event.target.name]: event.target.checked });
  };

  const handleSubmit = event => {
    event.preventDefault();
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
              md={6}
              sm={6}
              xs={6}
            >

              <Typography
                color="textPrimary"
                gutterBottom
                variant="h6"
              >
                Mining
              </Typography>
              <FormControl component="fieldset">
                {/* <FormLabel component="legend">Mining</FormLabel> */}
                <FormGroup>
                  <FormControlLabel
                    control={<Switch checked={state.auto_mining} onChange={handleChange} name="gilad" />}
                    label="Auto Mining"
                    disabled
                  />
                </FormGroup>
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
