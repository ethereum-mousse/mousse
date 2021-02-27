import React from 'react';
import { Navigate } from 'react-router-dom';
import DashboardLayout from 'src/layouts/DashboardLayout';
import MainLayout from 'src/layouts/MainLayout';
import BlockListView from 'src/views/blocks/BlockListView';
import FinalizedBlockListView from 'src/views/finalized_blocks/FinalizedBlockListView';
import NotFoundView from 'src/views/errors/NotFoundView';
import LogListView from 'src/views/logs/LogListView';
import ProcessingView from 'src/views/processing/ProcessingView';
import SettingsView from 'src/views/settings/SettingsView';

const routes = [
  {
    path: 'app',
    element: <DashboardLayout />,
    children: [
      { path: 'blocks', element: <BlockListView /> },
      { path: 'finalized_blocks', element: <FinalizedBlockListView /> },
      { path: 'logs', element: <LogListView /> },
      { path: 'settings', element: <SettingsView /> },
      { path: 'processing', element: <ProcessingView /> },
      { path: '*', element: <Navigate to="/404" /> }
    ]
  },
  {
    path: '/',
    element: <MainLayout />,
    children: [
      { path: '404', element: <NotFoundView /> },
      { path: '/', element: <Navigate to="/app/blocks" /> },
      { path: '*', element: <Navigate to="/404" /> }
    ]
  }
];

export default routes;
