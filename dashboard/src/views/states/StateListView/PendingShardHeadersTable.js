import React from 'react';
import PropTypes from 'prop-types';
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableRow,
    Typography,
    makeStyles,
} from '@material-ui/core';
import bytesToHex from 'src/utils/bytesToHex';
import omitString from 'src/utils/omitString';

const useStyles = makeStyles((theme) => ({
    monospace: {
        fontFamily: 'Consolas, "Liberation Mono", Menlo, Courier, monospace',
        backgroundColor: '#e7edf3',
        padding: '0.15em 0.3em'
    }
}));

const PendingShardHeadersTable = ({ className, pending_shard_headers, ...rest }) => {
    const classes = useStyles();

    return (
        <React.Fragment>
            {(pending_shard_headers && pending_shard_headers.length >= 1) ?
                <Table size="small" aria-label="purchases">
                    <TableHead>
                        <TableRow>
                            <TableCell align="right">SLOT</TableCell>
                            <TableCell align="right">SHARD</TableCell>
                            <TableCell>COMMITMENT</TableCell>
                            <TableCell>ROOT</TableCell>
                            <TableCell>CONFIRMED</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {pending_shard_headers.map((shard_header, index) => (
                            <TableRow key={index}>
                                <TableCell align="right">
                                    {shard_header.slot}
                                </TableCell>
                                <TableCell align="right">
                                    {shard_header.shard}
                                </TableCell>
                                <TableCell>
                                    Point: <code className={classes.monospace}>0x{omitString(bytesToHex(shard_header.commitment.point), 64)}</code><br />
                                      Length: {shard_header.commitment.length}
                                </TableCell>
                                <TableCell><code className={classes.monospace}>0x{omitString(shard_header.root, 64)}</code></TableCell>
                                <TableCell>{shard_header.confirmed ? "True" : "False"}</TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
                : <Typography>None</Typography>}
        </React.Fragment>
    );
};

PendingShardHeadersTable.propTypes = {
    className: PropTypes.string,
    states: PropTypes.array.isRequired
};

export default PendingShardHeadersTable;
