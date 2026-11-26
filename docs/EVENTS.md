# Events

Events are attached with attributes:

```rsml
<button on-click="increment">Increment</button>
<input on-input="update_name" on-change="save_name"/>
<form on-submit="login">
    <button type="submit">Login</button>
</form>
```

Supported event attributes:

- `on-click`
- `on-hover`
- `on-focus`
- `on-change`
- `on-submit`
- `on-input`
- `on-load`
- `on-scroll`
- `on-keydown`
- `on-keyup`

Handler names must be valid RUSK identifiers. Generated JavaScript dispatches them to `window.RuskApp[name]`. Markup does not execute arbitrary JavaScript.

## Native RUSK handlers

Associate a script with the window:

```rsml
<window title="Counter" script="main.rusk">
    <text bind="counter">0</text>
    <button on-click="increment">Increment</button>
</window>
```

The script may be alongside `index.rsml`:

```rusk
let counter be 0

function increment() {
    set counter to counter + 1
}
```

If `script` is omitted, `build-web` automatically looks for sibling `main.rusk`.

## Browser bridge

```html
<script>
window.RuskApp = {
  increment(payload) {
    RuskWeb.setState("count", (RuskWeb.state.count || 0) + 1);
  }
};
</script>
```

Binding:

```rsml
<text bind="count">0</text>
```
