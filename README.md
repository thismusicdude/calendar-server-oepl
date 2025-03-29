# Calendar Server for Open Epaper Link

This project provides a web application for managing and displaying events. The server exposes several APIs for setting and retrieving calendar configurations, as well as an endpoint to display the next events. It uses Actix-web and Tokio to handle HTTP requests and is designed to have an easy to use headless setup.

![image](https://github.com/user-attachments/assets/22a58103-bb1f-4785-9acd-1b2f1d018ee0)



## Setup

1. **Clone the Repository**

   First, clone the repository to your local machine:

   ```bash
   git clone <repository-url>
   cd <project-directory>
   ```

2. **Install Dependencies**

   Ensure you have Rust installed on your system. If not, follow the instructions at [Rust's official website](https://www.rust-lang.org/tools/install).

   After that, install the required dependencies:

   ```bash
   cargo build --release
   ```

3. **Run the Server**

   To start the server, use the following command:

   ```bash
   cargo run --release
   ```

   The server will be available at `http://localhost:8080`.

4. **Access the Setup Page**

   To configure the `config.json`, open the setup page in your browser:

   ```plaintext
   http://localhost:8080/setup
   ```

   This page allows you to input and set up the required configuration file.
   Click here to upload the file to the server.
   
  ![image](https://github.com/user-attachments/assets/5ffefc17-c22c-4ea5-bbfd-806cf2a03025)

## API Endpoints
1. **`GET /next_events.json`**  
   Get the next events, sorted by date.

   - Response: A JSON template for OpenEPaperLink with event details.
   
2. **`GET /setup`**  
   Provides an HTML page to configure the `config.json`.

   - Response: A simple form for configuring the settings.

3. **`POST /api/set_config`**  
   Set the calendar configuration.

   - Request body: `ServerConfigFile` in JSON format.
   - Example:
     ```json
       {
        "urls":[
            "https://example.com/calendar.ics",
            "https://localhost:8080/calendar.ics"
        ],
        "ammount_of_next_events": 2,
        "nothing_todo_message": "Done for Today!"
    }
     ```
   - Response: `"set confg successul"`

4. **`GET /api/get_config`**  
   Retrieve the current calendar configuration.

   - Response: `ServerConfigFile` in JSON format.
   
## config.json

The `config.json` file contains the settings for the server. Here's the structure of the configuration:

```json
  {
        "urls":[
            "https://example.com/calendar.ics",
            "https://localhost:8080/calendar.ics"
        ],
        "ammount_of_next_events": 2,
        "nothing_todo_message": "Done for Today!"
    }
```

### Fields

- **`urls`**: A list of URLs (strings) where the server will fetch events from.
- **`ammount_of_next_events`**: The number of upcoming events to retrieve and display.
- **`nothing_todo_message`**: A message to display when there are no upcoming events.

## Example Usage

1. Open the setup page: `http://localhost:8080/setup`
2. Configure your calendar URLs, the number of events to display, and the "no events" message.
3. The configuration will be saved to `config.json`.
4. Access the events via `http://localhost:8080/next_events.json`.

## Additional Notes
- The server is currently set to run on `0.0.0.0:8080`.
- Make sure the `config.json` file exists and is properly configured before using the API endpoints. You can either manually create it or use the setup page to do so.

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


