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
import GrandparentEpochConfirmedCommitmentsTable from './GrandparentEpochConfirmedCommitmentsTable';

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

  const [openedIds, setOpenedIds] = useState(new Set());

  const handleOpen = (id) => {
    let newOpenedIds = new Set(openedIds);
    if (newOpenedIds.has(id)) {
      newOpenedIds.delete(id);
    } else {
      newOpenedIds.add(id);
    }
    setOpenedIds(newOpenedIds);
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
                    onClick={() => handleOpen(index)}
                    className={stateColorClassName(state)}
                  >
                    <TableCell padding="checkbox" align="right">
                      <IconButton aria-label="expand row" size="small">
                        {openedIds.has(index) ? <KeyboardArrowDownIcon /> : <KeyboardArrowRightIcon />}
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
                      <Collapse in={openedIds.has(index)} timeout={300} unmountOnExit>
                        <Box margin={2}>
                          <Typography
                            variant="h5"
                            gutterBottom
                            component="div"
                            onClick={() => handleOpen(index + "current")}
                          >
                            <IconButton aria-label="expand row" size="small">
                              {openedIds.has(index + "current") ? <KeyboardArrowDownIcon /> : <KeyboardArrowRightIcon />}
                            </IconButton>
                            Current Epoch Pending Shard Headers
                          </Typography>

                          <Collapse in={openedIds.has(index + "current")} timeout={300} unmountOnExit>
                            <PendingShardHeadersTable pending_shard_headers={state.current_epoch_pending_shard_headers}></PendingShardHeadersTable>
                          </Collapse>
                          <Typography
                            variant="h5"
                            gutterBottom
                            component="div"
                            onClick={() => handleOpen(index + "previous")}
                          >
                            <IconButton aria-label="expand row" size="small">
                              {openedIds.has(index + "previous") ? <KeyboardArrowDownIcon /> : <KeyboardArrowRightIcon />}
                            </IconButton>
                            Previous Epoch Pending Shard Headers
                          </Typography>
                          <Collapse in={openedIds.has(index + "previous")} timeout={300} unmountOnExit>
                            <PendingShardHeadersTable pending_shard_headers={state.previous_epoch_pending_shard_headers}></PendingShardHeadersTable>
                          </Collapse>

                          <Typography
                            variant="h5"
                            gutterBottom
                            component="div"
                            onClick={() => handleOpen(index + "grandparent")}
                          >
                            <IconButton aria-label="expand row" size="small">
                              {openedIds.has(index + "grandparent") ? <KeyboardArrowDownIcon /> : <KeyboardArrowRightIcon />}
                            </IconButton>
                            Grandparent Epoch Confirmed Commitments
                          </Typography>
                          <Collapse in={openedIds.has(index + "grandparent")} timeout={300} unmountOnExit>
                            <GrandparentEpochConfirmedCommitmentsTable grandparent_epoch_confirmed_commitments={state.grandparent_epoch_confirmed_commitments}></GrandparentEpochConfirmedCommitmentsTable>
                          </Collapse>
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
