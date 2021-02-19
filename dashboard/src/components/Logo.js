import React from 'react';

const Logo = (props) => {
  return (
    <img
      alt="Logo"
      src="/static/logo_mousse.svg"
      width="45px"
      {...props}
    />
  );
};

export default Logo;
