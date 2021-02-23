export default (text, len, ellipsis = "...") =>
    text.length >= len
        ? text.slice(0, len - ellipsis.length) + ellipsis
        : text;
