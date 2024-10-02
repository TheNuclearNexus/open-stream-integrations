export function unflatten(obj) {
  if (typeof obj === "object") {
    let copy = {};

    for (const k in obj) {
      const child = obj[k];
      const keys = k.split(".");
      
      if (keys.length > 1) {
        const newKey = keys.slice(1).join(".")
        const newChild = { [newKey]: child, ...copy[keys[0]] };
        copy[keys[0]] = unflatten(newChild);
      } else {
        copy[keys[0]] = child;
      }
    }

    return copy;
  }
  return obj;
}