<script lang="ts">
  import QRCode from "qrcode";

  interface Props {
    name: string;
    url: string;
  }

  const { name, url }: Props = $props();

  const qrCodeImage = $derived.by(() => {
    try {
      return QRCode.toDataURL(url, { width: 220, margin: 1 });
    } catch (error) {
      console.error(error, "failed to generate QR code image");
      throw error;
    }
  });
</script>

<section class="invite" aria-label="Join this quiz">
  <div class="invite__qr-wrap">
    {#await qrCodeImage then qrImg}
      <img src={qrImg} alt="QR Code to join {name}" class="invite__qr" />
    {:catch}
      <p>QR code unavailable. Use the link to join.</p>
    {/await}
  </div>
  <div>
    <h3>Scan to join</h3>
    <p>Scan this code to open the quiz join page.</p>
    <a class="invite__link" href={url}>{url}</a>
  </div>
</section>

<style>
  .invite {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin: 1rem 0;
    padding: 1rem;
    border-radius: 0.5rem;
    background-color: var(--surface);
  }

  .invite__qr-wrap {
    flex: 0 0 11rem;
  }

  .invite__qr {
    display: block;
    width: 11rem;
    height: 11rem;
    background: white;
  }

  .invite__link {
    overflow-wrap: anywhere;
  }

  @media screen and (max-width: 32rem) {
    .invite {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
