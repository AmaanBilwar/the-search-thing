import { protocol, net } from "electron";
import { existsSync, statSync } from "fs";
import { join } from "path";
import { pathToFileURL } from "url";

export function registerResourcesProtocol() {
  protocol.handle("res", async (request) => {
    try {
      const url = new URL(request.url);
      // Combine hostname and pathname to get the full path
      const fullPath = join(url.hostname, url.pathname.slice(1));
      const filePath = join(__dirname, "../../resources", fullPath);
      return net.fetch(pathToFileURL(filePath).toString());
    } catch (error) {
      console.error("Protocol error:", error);
      return new Response("Resource not found", { status: 404 });
    }
  });

  protocol.handle("localimg", async (request) => {
    try {
      const url = new URL(request.url);
      const rawPath = url.searchParams.get("path");
      if (!rawPath) {
        return new Response("Missing image path", { status: 400 });
      }

      const decodedPath = rawPath;
      if (!existsSync(decodedPath)) {
        return new Response("Image not found", { status: 404 });
      }

      const stats = statSync(decodedPath);
      if (!stats.isFile()) {
        return new Response("Invalid image path", { status: 400 });
      }

      // Delegate content-type detection to Chromium via file:// fetch.
      return net.fetch(pathToFileURL(decodedPath).toString());
    } catch (error) {
      console.error("Local image protocol error:", error);
      return new Response("Image preview unavailable", { status: 404 });
    }
  });
}
