export default (hex) => {
  let arr = [];
  arr.push(...Buffer.from(hex, 'hex'));
  return arr;
};
