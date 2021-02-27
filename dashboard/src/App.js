import 'react-perfect-scrollbar/dist/css/styles.css';
import React, { useState } from 'react';
import { useRoutes } from 'react-router-dom';
import { ThemeProvider } from '@material-ui/core';
import GlobalStyles from 'src/components/GlobalStyles';
import 'src/mixins/chartjs';
import theme from 'src/theme';
import routes from 'src/routes';
import { CurrentSlotContext } from 'src/contexts/CurrentSlotContext';

const App = () => {
  const routing = useRoutes(routes);

  const [current_slot, setCurrentSlot] = useState(null);

  return (
    <ThemeProvider theme={theme}>
      <CurrentSlotContext.Provider value={{ current_slot: current_slot, setCurrentSlot: setCurrentSlot }}>
        <GlobalStyles />
        {routing}
      </CurrentSlotContext.Provider>
    </ThemeProvider >
  );
};

export default App;
