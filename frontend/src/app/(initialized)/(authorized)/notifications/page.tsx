import { apiUrl } from "@/lib/api";
import getAccessKeyOrRedirect from "@/lib/api/getAccessKeyOrRedirect";
import { Notification, throwError } from "@/lib/dto";
import * as z from "zod";

export const metadata = {
  title: "Notifications",
};

async function getNotifications() {
  const accessKey = await getAccessKeyOrRedirect();

  const resp = await fetch(apiUrl("/api/notification"), {
    headers: {
      "authorization": `Bearer ${accessKey}`,
    },
  });
  if (!resp.ok) {
    await throwError(resp);
  }
  return z.array(Notification).parse(await resp.json());
}

export default async function NotificationsPage() {
  const notifications = await getNotifications();
  return (
    <div>
      {notifications.map(notification => (
        <div key={notification.id}>{JSON.stringify(notification)}</div>
      ))}
    </div>
  );
}
