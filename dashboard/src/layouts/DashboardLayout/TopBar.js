import React, { useState, useEffect } from 'react';
import { Link as RouterLink } from 'react-router-dom';
import clsx from 'clsx';
import PropTypes from 'prop-types';
import axios from 'axios';
import {
  AppBar,
  Box,
  Typography,
  Toolbar,
  makeStyles
} from '@material-ui/core';
import Logo from 'src/components/Logo';

const useStyles = makeStyles((theme) => ({
  root: {},
  logo: {
    marginRight: theme.spacing(4)
  },
  info: {
    borderLeft: '0.3em solid',
    borderColor: '#D9BAA4',
    paddingLeft: '1em',
    paddingRight: '1em',
    textAlign: 'center'
  },
  info_value: {
    fontWeight: 'bold',
  }
}));

const TopBar = ({
  className,
  ...rest
}) => {
  const classes = useStyles();
  const [config, setConfig] = useState(null);

  useEffect(() => {
    const fetchData = async () => {
      const result = await axios(
        'http://localhost:' + process.env.REACT_APP_EMULATOR_PORT_NUMBER + '/utils/current_status_for_polling'
      );
      let data = result.data;

      if (data) {
        if (data.slot) {
          rest.setCurrentSlot(result.data.slot);
        }
        else {
          rest.setCurrentSlot(0);
        }
        setConfig(data.config);
      }
      else {
        rest.setCurrentSlot(0);
      }
    };

    const interval = setInterval(fetchData, 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <AppBar
      className={clsx(classes.root, className)}
      elevation={0}
      {...rest}
    >
      <Toolbar>
        <RouterLink to="/">
          <Logo className={classes.logo} />
        </RouterLink>
        <Box className={classes.info}>
          <Typography variant="h6">
            CURRENT SLOT
          </Typography>
          <Typography variant="h4" className={classes.info_value}>
            {rest.current_slot === null ? 0 : rest.current_slot}
          </Typography>
        </Box>
        <Box className={classes.info}>
          <Typography variant="h6">
            SERVER
          </Typography>
          <Typography variant="h4" className={classes.info_value}>
            http://localhost:{process.env.REACT_APP_EMULATOR_PORT_NUMBER}
          </Typography>
        </Box>
        <Box className={classes.info}>
          <Typography variant="h6">
            SIMULATOR MODE
          </Typography>
          <Typography variant="h4" className={classes.info_value}>
            {config && config.auto ? "AUTO" : "MANUAL"}
          </Typography>
        </Box>
      </Toolbar>
    </AppBar>
  );
};

TopBar.propTypes = {
  className: PropTypes.string,
};

export default TopBar;
