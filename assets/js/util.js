const csrfToken = function() {
  const node = document.head.querySelector('meta[name="csrf"]');
  const token = node.content;
  return token;
};
export { csrfToken };
