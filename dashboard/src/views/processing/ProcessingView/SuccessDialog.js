import React from 'react';
import Button from '@material-ui/core/Button';
import Dialog from '@material-ui/core/Dialog';
import DialogActions from '@material-ui/core/DialogActions';
import DialogContent from '@material-ui/core/DialogContent';
import DialogContentText from '@material-ui/core/DialogContentText';
import DialogTitle from '@material-ui/core/DialogTitle';

export default function AlertDialog({ ...rest }) {
    return (
        <div>
            <Dialog
                open={rest.success_open}
                onClose={() => rest.setSuccessOpen(false)}
                aria-labelledby="alert-dialog-title"
                aria-describedby="alert-dialog-description"
            >
                {/* <DialogTitle id="alert-dialog-title">Success</DialogTitle> */}
                <DialogContent>
                    <DialogContentText id="alert-dialog-description">
                        Success
                    </DialogContentText>
                </DialogContent>
                <DialogActions>
                    <Button onClick={() => rest.setSuccessOpen(false)} color="primary" autoFocus>
                        OK
                    </Button>
                </DialogActions>
            </Dialog>
        </div>
    );
}