import getAccessKeyOrRedirect from "@/lib/api/getAccessKeyOrRedirect";

export const metadata = {
  title: "Notifications",
};

export default async function NotificationsPage() {
  await getAccessKeyOrRedirect();

  return (
    <div>
      Notifications
    </div>
  );
}
