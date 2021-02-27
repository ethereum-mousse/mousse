import React, { useState } from 'react';
import clsx from 'clsx';
import PropTypes from 'prop-types';
// import moment from 'moment';
import PerfectScrollbar from 'react-perfect-scrollbar';
import {
  Box,
  Card,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TablePagination,
  TableRow,
  makeStyles
} from '@material-ui/core';
// import getInitials from 'src/utils/getInitials';

const useStyles = makeStyles((theme) => ({
  root: {},
  avatar: {
    marginRight: theme.spacing(2)
  }
}));

const Results = ({ className, logs, ...rest }) => {
  const classes = useStyles();
  const [limit, setLimit] = useState(10);
  const [page, setPage] = useState(0);

  const handleLimitChange = (event) => {
    setLimit(event.target.value);
  };

  const handlePageChange = (event, newPage) => {
    setPage(newPage);
  };

  return (
    <Card
      className={clsx(classes.root, className)}
      {...rest}
    >
      <PerfectScrollbar>
        <Box minWidth={1050}>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell align="right">
                  LOG ID
                </TableCell>
                <TableCell>
                  DATE
                </TableCell>
                <TableCell>
                  ENDPOINT
                </TableCell>
                {/* <TableCell>
                  REQUEST BODY
                </TableCell>
                <TableCell>
                  RESPONSE BODY
                </TableCell> */}
              </TableRow>
            </TableHead>
            <TableBody>
              {logs.slice(page * limit, (page + 1) * limit).map((log) => (
                <TableRow
                  hover
                  key={log.log_id}
                >
                  <TableCell align="right">
                    {log.log_id}
                  </TableCell>
                  <TableCell>
                    {/* {moment(log.date).format('YYYY-MM-DD HH:mm:ss')} */}
                    {log.date}
                  </TableCell>
                  <TableCell>
                    {log.endpoint}
                  </TableCell>
                  {/* <TableCell>
                    -
                  </TableCell>
                  <TableCell>
                    -
                  </TableCell> */}
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Box>
      </PerfectScrollbar>
      <TablePagination
        component="div"
        count={logs.length}
        onChangePage={handlePageChange}
        onChangeRowsPerPage={handleLimitChange}
        page={page}
        rowsPerPage={limit}
        rowsPerPageOptions={[5, 10, 25, 50, 100]}
      />
    </Card>
  );
};

Results.propTypes = {
  className: PropTypes.string,
  logs: PropTypes.array.isRequired
};

export default Results;
