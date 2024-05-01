# 🐕 hasskey

Hasskey forwards keyboard interactions to [Home Assistant](https://www.home-assistant.io/).

Hasskey is a small daemon that listens for keyboard (or any other HID key) events and forward these events to the Home Assistant API.
It supports watching multiple input devices with a single instance.
On each keyboard event, hasskey will create a Home Assistant Event using the REST API.

> [!NOTE]
> Hasskey currently is linux only.

## 🔧 Configuration
Hasskey reads a configuration file on startup.
The default config file path is `config.yaml` and can be changed using a command line option.
The config file uses the YAML file format and consists of the following sections:

### Home Assistant
The `home-assistant` section defines which Home Assistant instance to talk to.
The `url` field defines the base URL of your Home Assistant instance.
The `token` field must contain a valid [long-lived access token](https://www.home-assistant.io/docs/authentication/#your-account-profile).

### Devices
The `devices` section contains a list of input devices to monitor for events.
Each entry consists of the `name` field containing a unique name for the device to monitor.
This name is used to identify the device in the created Home Assistant events.

To identify the device, Hasskey provides multiple options where only one can be used at a time.
- The `input` field is used to search for the input device with the given input name.
- The `path` field is used to identify the input by the device path.
- The `bus_type`, `vendor`, `product` and `version` fields are used to identify the device by the provided device metadata.

### Example
```yaml
home-assistant:
  url: https://my.home-assistant.instance/
  token: a-very-long-access-token

devices:
  - name: my-device
    input: My Input Device
  - name: another-device
    path: /dev/input/input-9999
```

## 🌠 Home Assistant Events
Hasskey creates an event in Home Assistant for every key-press detected.
The event has the event type `hasskey` and contains the following data:
- `device`: The name of the device as specified in the config using the `name` field.
- `key`: The name of the key pressed.

## 🤝 Contributing
We welcome contributions from the community to help improve Hasskey.
Whether you're a developer, designer, or enthusiast, there are many ways to get involved:

* **Bug Reports:** Report any issues or bugs you encounter while using Hasskey.
* **Feature Requests:** Suggest new features or enhancements to make Hasskey even more powerful.
* **Pull Requests:** Submit pull requests to address bugs, implement new features, or improve documentation.

## 📄 License
Hasskey is licensed under the MIT License, which means you are free to use, modify, and distribute the software for both commercial and non-commercial purposes. See the [LICENSE](./LICENSE) file for more details.

## 🛟 Support
If you have any questions, concerns, or feedback about Hasskey, please [contact us](mailto:fooker@lab.sh) or open an issue on the project's GitHub repository.

## 🙏 Acknowledgements
We would like to thank all contributors and supporters who have helped make Hasskey possible. Your contributions and feedback are greatly appreciated!

