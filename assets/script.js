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
  const sub_name_field = document.getElementById('subscriptionName');
  if (!sub_name_field.reportValidity()) {
    return;
  }

  const sub_data = {
    userVisibleOnly: true,
    applicationServerKey: await fetch('/vapid/pubkey')
      .then(resp => resp.json())
      .then(json => json.vapid_public_key)
  };
  const subscription = await registration.pushManager.subscribe(sub_data);
  await updateButtons();
  const sub = subscription.toJSON();
  sub.name = sub_name_field.value;

  const resp = await fetch('/subscribe', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(sub)
  });
  sub_name_field.value = "";
  return resp
}

async function unsubscribeFromPush() {
  const registration = await navigator.serviceWorker.getRegistration();
  if (registration === undefined) return;

  const subscription = await registration.pushManager.getSubscription();
  if (subscription === null) return;

  await subscription.unsubscribe();
  await updateButtons();

  const endpoint_query = new URLSearchParams({ endpoint: subscription.endpoint });
  await fetch(`/unsubscribe?${endpoint_query}`, {
    method: 'DELETE'
  });
}

async function checkTestPush() {
  return await fetch('/test-push/info')
    .then(resp => resp.json())
    .then(json => json.exists)
}

async function updateButtons() {
  const reg_button = document.getElementById('register');
  const sub_button = document.getElementById('subscribe');
  const sub_name_field = document.getElementById('subscriptionName');
  const test_button = document.getElementById('testButton');
  const message_field = document.getElementById('testMessage');

  const registration = await navigator.serviceWorker.getRegistration();
  const registered = registration !== undefined;
  reg_button.onclick = registered ? unregisterServiceWorker : registerServiceWorker;
  reg_button.textContent = registered ? "unregister" : "register";
  test_button.disabled = !registered;
  sub_button.disabled = !registered;
  sub_name_field.hidden = !registered;
  message_field.hidden = !registered;
  if (!registered) return;

  const subscription = await registration.pushManager.getSubscription();
  const subscribed = subscription !== null;
  sub_button.textContent = subscribed ? "unsubscribe" : "subscribe";
  sub_button.onclick = subscribed ? unsubscribeFromPush : subscribeToPush
  sub_name_field.hidden = subscribed;
  test_button.disabled = !subscribed;
  message_field.hidden = !subscribed;
  if (!test_button.disabled) {
    test_button.disabled = !await checkTestPush();
  }
  if (!test_button.disabled) {
    test_button.onclick = testPush
  }
}

async function testPush() {
  const message = document.getElementById('testMessage');
  const resp = await fetch('/test-push', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message: message.value })
  });
  message.value = ""
  return resp
}

window.onload = updateButtons;
