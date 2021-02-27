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
import bytesToHex from 'src/utils/bytesToHex';
import omitString from 'src/utils/omitString';

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

const Results = ({ className, blocks, ...rest }) => {
  const classes = useStyles();
  const [openedBlockIds, setOpenedBlockIds] = useState(new Set());
  const [limit, setLimit] = useState(10);
  const [page, setPage] = useState(0);

  const handleOpenBlock = (id) => {
    let newOpenedBlockIds = new Set(openedBlockIds);
    if (newOpenedBlockIds.has(id)) {
      newOpenedBlockIds.delete(id);
    } else {
      newOpenedBlockIds.add(id);
    }
    setOpenedBlockIds(newOpenedBlockIds);
  }

  const handleLimitChange = (event) => {
    setLimit(event.target.value);
  };

  const handlePageChange = (event, newPage) => {
    setPage(newPage);
  };

  const blockColorClassName = block => {
    if (block.state_root == null || block.shard_headers.length === 0) {
      return classes.table_row_red;
    }
    else {
      return classes.table_row;
    }
  };

  let slot_to_block = [];
  if (blocks.length > 0) {
    let block_id = 0;
    for (let slot = 0; slot <= blocks[blocks.length - 1].slot; slot++) {
      if (slot === blocks[block_id].slot) {
        slot_to_block.push(blocks[block_id]);
        block_id += 1;
      }
      else {
        slot_to_block.push({
          slot: slot,
          parent_root: null,
          state_root: null,
          shard_headers: null,
        });
      }
    }
  }
  let slot_to_block_rev = slot_to_block;
  slot_to_block_rev.reverse();

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
                  SHARD HEADERS COUNT
                </TableCell>
                <TableCell>
                  STATE ROOT
                </TableCell>
                {/* <TableCell>
                  CREATED AT
                </TableCell> */}
              </TableRow>
            </TableHead>
            <TableBody>
              {slot_to_block_rev.slice(page * limit, (page + 1) * limit).map((block, index) => (
                <React.Fragment
                  key={index}>
                  <TableRow
                    hover
                    onClick={() => handleOpenBlock(index)}
                    className={blockColorClassName(block)}
                  >
                    <TableCell padding="checkbox" align="right">
                      <IconButton aria-label="expand row" size="small">
                        {openedBlockIds.has(index) ? <KeyboardArrowDownIcon /> : <KeyboardArrowRightIcon />}
                      </IconButton>
                    </TableCell>
                    <TableCell align="right">
                      {block.slot}
                    </TableCell>
                    <TableCell align="right">
                      {block.state_root ? block.shard_headers.length : "-"}
                    </TableCell>
                    <TableCell>
                      <code className={classes.monospace}>{block.state_root ? block.state_root : ""}</code>
                    </TableCell>
                    {/* <TableCell>
                      {moment(block.createdAt).format('YYYY-MM-DD HH:mm:ss')}
                    </TableCell> */}
                  </TableRow>
                  <TableRow>
                    <TableCell style={{ paddingBottom: 0, paddingTop: 0 }} colSpan={5}>
                      <Collapse in={openedBlockIds.has(index)} timeout={300} unmountOnExit>
                        <Box margin={2}>
                          <Typography variant="h5" gutterBottom component="div">
                            Shard Headers
                          </Typography>
                          {(block.state_root && block.shard_headers.length >= 1) ?
                            <Table size="small" aria-label="purchases">
                              <TableHead>
                                <TableRow>
                                  <TableCell align="right">SLOT</TableCell>
                                  <TableCell align="right">SHARD</TableCell>
                                  <TableCell>COMMITMENT</TableCell>
                                  <TableCell>SIGNATURE</TableCell>
                                </TableRow>
                              </TableHead>
                              <TableBody>
                                {block.shard_headers.map((shard_header, index) => (
                                  <TableRow key={index}>
                                    <TableCell align="right">
                                      {shard_header.message.slot}
                                    </TableCell>
                                    <TableCell align="right">
                                      {shard_header.message.shard}
                                    </TableCell>
                                    <TableCell>
                                      Point: <code className={classes.monospace}>0x{omitString(bytesToHex(shard_header.message.commitment.point), 64)}</code><br />
                                      Length: {shard_header.message.commitment.length}
                                    </TableCell>
                                    <TableCell><code className={classes.monospace}>0x{omitString(bytesToHex(shard_header.signature), 64)}</code></TableCell>
                                  </TableRow>
                                ))}
                              </TableBody>
                            </Table>
                            : <Typography>None</Typography>}
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
        count={blocks.length}
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
  blocks: PropTypes.array.isRequired
};

export default Results;
