async function registerServiceWorker() {
  await navigator.serviceWorker.register('./sw.js');
  await updateButtons();
}

async function unregisterServiceWorker() {
  const registration = await navigator.serviceWorker.getRegistration();
  if (registration === undefined) return;

  await registration.unregister();
  await updateButtons();
}

async function subscribeToPush() {
  const registration = await navigator.serviceWorker.getRegistration();
  if (registration === undefined) return;

  const sub_data = {
    userVisibleOnly: true,
    applicationServerKey: await fetch('/vapid/pubkey')
      .then(resp => resp.json())
      .then(json => json.vapid_public_key)
  };
  const subscription = await registration.pushManager.subscribe(sub_data);
  await updateButtons();

  await fetch('/subscribe', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(subscription.toJSON())
  });
}

async function unsubscribeFromPush() {
  const registration = await navigator.serviceWorker.getRegistration();
  if (registration === undefined) return;

  const subscription = await registration.pushManager.getSubscription();
  if (subscription === null) return;

  const sub_json = subscription.toJSON();
  await subscription.unsubscribe();
  await updateButtons();

  await fetch('/unsubscribe', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(sub_json)
  });
}

async function updateButtons() {
  const reg_button = document.getElementById('register');
  const unreg_button = document.getElementById('unregister');
  const sub_button = document.getElementById('subscribe');
  const unsub_button = document.getElementById('unsubscribe');

  const registration = await navigator.serviceWorker.getRegistration();
  const registered = registration !== undefined;
  unreg_button.disabled = !registered;
  sub_button.disabled = !registered;
  unsub_button.disabled = !registered;
  reg_button.disabled = registered;
  if (!registered) return;

  const subscription = await registration.pushManager.getSubscription();
  const subscribed = subscription !== null;
  sub_button.disabled = subscribed;
  unsub_button.disabled = !subscribed;
}

async function testPush() {
  await fetch('/test-push', { method: 'POST' });
}

window.onload = updateButtons;
