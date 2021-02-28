import React from 'react';
import { Link as RouterLink, useLocation } from 'react-router-dom';
// import PropTypes from 'prop-types';
import {
  Box,
  Drawer,
  List,
  makeStyles
} from '@material-ui/core';
import {
  Copy as BlocksIcon,
  Database as StatesIcon,
  FastForward as ProcessingIcon,
  Archive as LogsIcon,
  Settings as SettingsIcon,
  // BarChart as BarChartIcon,
} from 'react-feather';
import NavItem from './NavItem';

const items = [
  {
    href: '/app/blocks',
    icon: BlocksIcon,
    title: 'Blocks'
  },
  {
    href: '/app/finalized_blocks',
    icon: BlocksIcon,
    title: 'Finalized Blocks'
  },
  {
    href: '/app/states',
    icon: StatesIcon,
    title: 'States'
  },
  {
    href: '/app/processing',
    icon: ProcessingIcon,
    title: 'Processing'
  },
  {
    href: '/app/logs',
    icon: LogsIcon,
    title: 'Logs'
  },
  {
    href: '/app/settings',
    icon: SettingsIcon,
    title: 'Settings'
  }
];

const useStyles = makeStyles(() => ({
  desktopDrawer: {
    width: 192,
    top: 64,
    height: 'calc(100% - 64px)'
  },
  avatar: {
    cursor: 'pointer',
    width: 64,
    height: 64
  }
}));

const NavBar = () => {
  const classes = useStyles();
  // const location = useLocation();

  const content = (
    <Box
      height="100%"
      display="flex"
      flexDirection="column"
    >
      <Box p={2}>
        <List>
          {items.map((item) => (
            <NavItem
              href={item.href}
              key={item.title}
              title={item.title}
              icon={item.icon}
            />
          ))}
        </List>
      </Box>
    </Box>
  );

  return (
    <>
      <Drawer
        anchor="left"
        classes={{ paper: classes.desktopDrawer }}
        open
        variant="persistent"
      >
        {content}
      </Drawer>
    </>
  );
};

export default NavBar;
