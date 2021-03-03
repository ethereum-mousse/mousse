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

const GrandparentEpochConfirmedCommitmentsTable = ({ className, grandparent_epoch_confirmed_commitments, ...rest }) => {
    const classes = useStyles();

    return (
        <React.Fragment>
            {(grandparent_epoch_confirmed_commitments && grandparent_epoch_confirmed_commitments.length >= 1) ?
                <Table size="small" aria-label="purchases">
                    <TableHead>
                        <TableRow>
                            <TableCell align="right">SHARD</TableCell>
                            <TableCell align="right">SLOT IN EPOCH</TableCell>
                            <TableCell>COMMITMENT</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {grandparent_epoch_confirmed_commitments.map((commitments, shard) =>
                            commitments.map((commitment, slot_in_epoch) => (
                                <TableRow key={shard + "," + slot_in_epoch}>
                                    <TableCell align="right">
                                        {shard}
                                    </TableCell>
                                    <TableCell align="right">
                                        {slot_in_epoch}
                                    </TableCell>
                                    <TableCell>
                                        Point: <code className={classes.monospace}>0x{omitString(bytesToHex(commitment.point), 64)}</code><br />
                                        Length: {commitment.length}
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
                : <Typography>None</Typography>}
        </React.Fragment>
    );
};

GrandparentEpochConfirmedCommitmentsTable.propTypes = {
    className: PropTypes.string,
    states: PropTypes.array.isRequired
};

export default GrandparentEpochConfirmedCommitmentsTable;
