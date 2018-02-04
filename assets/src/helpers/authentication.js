export function getCsrfToken() {
  const metas = document.getElementsByTagName('meta');

  for (let i = 0; i < metas.length; i++) {
    if (metas[i].getAttribute('name') == 'csrf') {
      return metas[i].getAttribute('content');
    }
  }
  return '';
}
