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
  // BarChart as BarChartIcon,
  Settings as SettingsIcon,
  Inbox as LogIcon,
  Copy as BlockIcon,
  FastForward as ProcessingIcon,
} from 'react-feather';
import NavItem from './NavItem';

const items = [
  {
    href: '/app/blocks',
    icon: BlockIcon,
    title: 'Blocks'
  },
  {
    href: '/app/finalized_blocks',
    icon: BlockIcon,
    title: 'Finalized Blocks'
  },
  {
    href: '/app/processing',
    icon: ProcessingIcon,
    title: 'Processing'
  },
  {
    href: '/app/logs',
    icon: LogIcon,
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
