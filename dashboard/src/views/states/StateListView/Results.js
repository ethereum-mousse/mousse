import React, { useState, useEffect } from 'react';
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

const Results = ({ className, ...rest }) => {
  const classes = useStyles();

  const [openedStateIds, setOpenedStateIds] = useState(new Set());

  const handleOpenState = (id) => {
    let newOpenedStateIds = new Set(openedStateIds);
    if (newOpenedStateIds.has(id)) {
      newOpenedStateIds.delete(id);
    } else {
      newOpenedStateIds.add(id);
    }
    setOpenedStateIds(newOpenedStateIds);
  }

  const handleCountChange = (event) => {
    let count = event.target.value;
    rest.setCount(count);
    rest.updateStates(count, rest.page);
  };

  const handlePageChange = (event, newPage) => {
    rest.setPage(newPage);
    rest.updateStates(rest.count, newPage);
  }

  const stateColorClassName = state => {
    // TODO: Remove
    return classes.table_row;
  };

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
              {rest.states.slice(0, rest.count).map((state, index) => (
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
        count={rest.current_slot}
        onChangePage={handlePageChange}
        onChangeRowsPerPage={handleCountChange}
        page={rest.page}
        rowsPerPage={rest.count}
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
