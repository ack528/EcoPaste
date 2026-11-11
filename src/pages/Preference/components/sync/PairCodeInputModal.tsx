import { Input, Modal } from "antd";
import type { FC } from "react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { syncPairWithCode } from "@/commands";
import { getMessageApi } from "@/utils/feedback";
import { log } from "@/utils/log";

interface PairCodeInputModalProps {
  open: boolean;
  onClose: () => void;
}

/**
 * 输入另一台设备上展示的 6 位匹配码，输满即自动提交。
 */
const PairCodeInputModal: FC<PairCodeInputModalProps> = (props) => {
  const { t } = useTranslation("preferences");
  const { open, onClose } = props;
  const [code, setCode] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!open) {
      setCode("");
      setLoading(false);
    }
  }, [open]);

  const submit = async (value: string) => {
    setLoading(true);

    try {
      const peer = await syncPairWithCode(value);

      getMessageApi().success(t("sync.pairing.success", { name: peer.name }));
      onClose();
    } catch (error) {
      log.warn("pair with code failed", error);
      setCode("");
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (value: string) => {
    setCode(value);

    if (value.length === 6) {
      void submit(value);
    }
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

        <Input.OTP
          autoFocus
          disabled={loading}
          formatter={formatDigits}
          length={6}
          onChange={handleChange}
          size="large"
          value={code}
        />
      </div>
    </Modal>
  );
};

export default PairCodeInputModal;
