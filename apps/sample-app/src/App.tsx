import { AuthProvider } from "@descope/react-sdk";
import Home from "./Home";

// Point the Descope SDK at the local emulator when running in dev/test.
// REACT_APP_DESCOPE_BASE_URL controls both the auth API and JWKS endpoints.
const BASE_URL =
  process.env.REACT_APP_DESCOPE_BASE_URL || "https://api.descope.com";
const STATIC_URL =
  process.env.REACT_APP_DESCOPE_BASE_STATIC_URL || BASE_URL;

const App = () => (
  <AuthProvider
    projectId={process.env.REACT_APP_DESCOPE_PROJECT_ID ?? ""}
    baseUrl={BASE_URL}
    baseStaticUrl={STATIC_URL}
  >
    <Home />
  </AuthProvider>
);

export default App;
