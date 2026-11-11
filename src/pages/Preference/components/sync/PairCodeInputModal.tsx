import { Button, Input, Modal } from "antd";
import type { ChangeEvent, FC } from "react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { syncPairWithCode } from "@/commands";
import { getMessageApi } from "@/utils/feedback";
import { log } from "@/utils/log";

const CODE_LENGTH = 6;

interface PairCodeInputModalProps {
  open: boolean;
  onClose: () => void;
}

/**
 * 输入另一台设备上展示的 6 位匹配码，输满即自动提交。
 * 局域网发现被网络拦截时，可展开填写对方 IP 直接连接。
 */
const PairCodeInputModal: FC<PairCodeInputModalProps> = (props) => {
  const { t } = useTranslation("preferences");
  const { open, onClose } = props;
  const [code, setCode] = useState("");
  const [host, setHost] = useState("");
  const [manual, setManual] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!open) {
      setCode("");
      setHost("");
      setManual(false);
      setLoading(false);
    }
  }, [open]);

  const submit = async (value: string) => {
    setLoading(true);

    try {
      const peer = await syncPairWithCode(
        value,
        manual && host.trim().length > 0 ? host.trim() : void 0,
      );

      getMessageApi().success(t("sync.pairing.success", { name: peer.name }));
      onClose();
    } catch (error) {
      log.warn("pair with code failed", error);
      setCode("");
    } finally {
      setLoading(false);
    }
  };

  const handleCodeChange = (value: string) => {
    setCode(value);

    if (value.length === CODE_LENGTH) {
      void submit(value);
    }
  };

  const handleHostChange = (event: ChangeEvent<HTMLInputElement>) => {
    setHost(event.target.value);
  };

  const handleHostEnter = () => {
    if (code.length !== CODE_LENGTH) return;

    void submit(code);
  };

  const showManual = () => {
    setManual(true);
  };

  const handleCancel = () => {
    if (loading) return;

    onClose();
  };

  const formatDigits = (value: string) => {
    return value.replace(/\D/g, "");
  };

  return (
    <Modal
      centered
      footer={null}
      onCancel={handleCancel}
      open={open}
      title={t("sync.input.title")}
      width={420}
    >
      <div className="flex flex-col items-center gap-4 py-2">
        <div className="text-center text-ant-secondary text-sm leading-relaxed">
          {t("sync.input.hint")}
        </div>

        {manual && (
          <div className="flex w-full flex-col gap-1.5">
            <span className="text-ant-secondary text-xs">
              {t("sync.input.hostLabel")}
            </span>
            <Input
              allowClear
              autoFocus
              disabled={loading}
              onChange={handleHostChange}
              onPressEnter={handleHostEnter}
              placeholder={t("sync.input.hostPlaceholder")}
              value={host}
            />
          </div>
        )}

        <Input.OTP
          autoFocus={!manual}
          disabled={loading}
          formatter={formatDigits}
          length={CODE_LENGTH}
          onChange={handleCodeChange}
          size="large"
          value={code}
        />

        {manual ? (
          <span className="text-center text-ant-secondary text-xs leading-relaxed">
            {t("sync.input.hostHint")}
          </span>
        ) : (
          <Button onClick={showManual} size="small" type="link">
            {t("sync.input.manual")}
          </Button>
        )}
      </div>
    </Modal>
  );
};

export default PairCodeInputModal;
