import React, { useState } from 'react';
import clsx from 'clsx';
import PropTypes from 'prop-types';
// import moment from 'moment';
import PerfectScrollbar from 'react-perfect-scrollbar';
import {
  Box,
  Card,
  Collapse,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TablePagination,
  TableRow,
  Typography,
  makeStyles,
  colors,
} from '@material-ui/core';
import IconButton from '@material-ui/core/IconButton';
import KeyboardArrowDownIcon from '@material-ui/icons/KeyboardArrowDown';
import KeyboardArrowRightIcon from '@material-ui/icons/KeyboardArrowRight';
import omitString from 'src/utils/omitString';
import PendingShardHeadersTable from './PendingShardHeadersTable';

const useStyles = makeStyles((theme) => ({
  root: {},
  table_row: {
    '& > *': {
      borderBottom: 'unset',
    },
  },
  table_row_red: {
    '& > *': {
      borderBottom: 'unset',
    },
    backgroundColor: colors.red[50]
  },
  monospace: {
    fontFamily: 'Consolas, "Liberation Mono", Menlo, Courier, monospace',
    backgroundColor: '#e7edf3',
    padding: '0.15em 0.3em'
  }
}));

const Results = ({ className, states, ...rest }) => {
  const classes = useStyles();
  const [openedStateIds, setOpenedStateIds] = useState(new Set());
  const [limit, setLimit] = useState(10);
  const [page, setPage] = useState(0);

  const handleOpenState = (id) => {
    let newOpenedStateIds = new Set(openedStateIds);
    if (newOpenedStateIds.has(id)) {
      newOpenedStateIds.delete(id);
    } else {
      newOpenedStateIds.add(id);
    }
    setOpenedStateIds(newOpenedStateIds);
  }

  const handleLimitChange = (event) => {
    setLimit(event.target.value);
  };

  const handlePageChange = (event, newPage) => {
    setPage(newPage);
  };

  const stateColorClassName = state => {
    // if (state.state_root == null || state.shard_headers.length === -1) {
    //   return classes.table_row_red;
    // }
    // else {
    return classes.table_row;
    // }
  };

  let slot_to_state = [];
  if (states.length > 0) {
    let state_id = 0;
    for (let slot = 0; slot <= states[states.length - 1].slot; slot++) {
      if (slot === states[state_id].slot) {
        slot_to_state.push(states[state_id]);
        state_id += 1;
      }
      else {
        slot_to_state.push({
          slot: slot,
          finalized_checkpoint: null,
          previous_epoch_pending_shard_headers: null,
          current_epoch_pending_shard_headers: null,
          shard_gasprice: null,
        });
      }
    }
  }
  let slot_to_state_rev = slot_to_state;
  slot_to_state_rev.reverse();

  return (
    <Card
      className={clsx(classes.root, className)}
      {...rest}
    >
      <PerfectScrollbar>
        <Box>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell padding="checkbox"></TableCell>
                <TableCell align="right">
                  SLOT
                </TableCell>
                <TableCell align="right">
                  SHARD GASPRICE
                </TableCell>
                <TableCell>
                  FINALIZED CHECKPOINT
                </TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {slot_to_state_rev.slice(page * limit, (page + 1) * limit).map((state, index) => (
                <React.Fragment
                  key={index}>
                  <TableRow
                    hover
                    onClick={() => handleOpenState(index)}
                    className={stateColorClassName(state)}
                  >
                    <TableCell padding="checkbox" align="right">
                      <IconButton aria-label="expand row" size="small">
                        {openedStateIds.has(index) ? <KeyboardArrowDownIcon /> : <KeyboardArrowRightIcon />}
                      </IconButton>
                    </TableCell>
                    <TableCell align="right">
                      {state.slot}
                    </TableCell>
                    <TableCell align="right">
                      {state.shard_gasprice} Gwei
                    </TableCell>
                    <TableCell>
                      Epoch: {state.finalized_checkpoint.epoch}<br />
                      Root: <code className={classes.monospace}>{omitString(state.finalized_checkpoint.root, 64)}</code>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell style={{ paddingBottom: 0, paddingTop: 0 }} colSpan={5}>
                      <Collapse in={openedStateIds.has(index)} timeout={300} unmountOnExit>
                        <Box margin={2}>
                          <Typography variant="h5" gutterBottom component="div">
                            Current Epoch Pending Shard Headers
                          </Typography>
                          <PendingShardHeadersTable pending_shard_headers={state.current_epoch_pending_shard_headers}></PendingShardHeadersTable>
                          <Typography variant="h5" gutterBottom component="div">
                            Previous Epoch Pending Shard Headers
                          </Typography>
                          <PendingShardHeadersTable pending_shard_headers={state.previous_epoch_pending_shard_headers}></PendingShardHeadersTable>
                        </Box>
                      </Collapse>
                    </TableCell>
                  </TableRow>
                </React.Fragment>
              ))}
            </TableBody>
          </Table>
        </Box>
      </PerfectScrollbar>
      <TablePagination
        component="div"
        count={states.length}
        onChangePage={handlePageChange}
        onChangeRowsPerPage={handleLimitChange}
        page={page}
        rowsPerPage={limit}
        rowsPerPageOptions={[5, 10, 25, 50, 100]}
      />
    </Card >
  );
};

Results.propTypes = {
  className: PropTypes.string,
  states: PropTypes.array.isRequired
};

export default Results;
