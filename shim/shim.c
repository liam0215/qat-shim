#include <stdint.h>
#include <stdlib.h>
#include "shim.h"

#define MAX_INSTANCES 1024

int qat_start_session(const char *pProcessName) {
  CpaStatus status = icp_sal_userStart(pProcessName);
  return (int) status;
}

int qat_stop_session() {
  CpaStatus status = icp_sal_userStop();
  if (status != CPA_STATUS_SUCCESS) {
    PRINT_ERR("Error stopping QAT session\n");
  }
  return (int) status;
}

int qat_qae_mem_init() {
  CpaStatus status = qaeMemInit();
  if (status != CPA_STATUS_SUCCESS) {
    PRINT_ERR("Error initializing memory for QAT\n");
  }
  return (int) status;
}

void qat_qae_mem_destroy() {
  qaeMemDestroy();
}

int qat_get_instance(CpaInstanceHandle *out) {
  CpaInstanceHandle instHandles[MAX_INSTANCES];
  Cpa16U numInstances = 0;
  CpaStatus status = CPA_STATUS_SUCCESS;
  CpaAccelerationServiceType accelSrvType = CPA_ACC_SVC_TYPE_CRYPTO;

  status = cpaGetNumInstances(accelSrvType, &numInstances);
  // status = cpaCyGetNumInstances(&numInstances);
  // if (status != CPA_STATUS_SUCCESS || numInstances == 0) {
  //   PRINT_ERR("Error getting QAT instances\n");
  // }
  //
  // status = cpaCyGetInstances(numInstances, instHandles);
  // if (status != CPA_STATUS_SUCCESS) {
  //   PRINT_ERR("Error retrieving QAT instances\n");
  // }
  // *out = instHandles[0];
  //
  if (0 == numInstances && (accelSrvType == CPA_ACC_SVC_TYPE_CRYPTO_SYM ||
                            accelSrvType == CPA_ACC_SVC_TYPE_CRYPTO_ASYM)) {
    accelSrvType = CPA_ACC_SVC_TYPE_CRYPTO;
    status = cpaGetNumInstances(accelSrvType, &numInstances);
  }
  if (numInstances > MAX_INSTANCES) {
    numInstances = MAX_INSTANCES;
  }
  if (0 == numInstances) {
    PRINT_ERR("No crypto instances found.\n");
  }
  if (status == CPA_STATUS_SUCCESS) {
    status = cpaGetInstances(accelSrvType, numInstances, instHandles);
    if (status == CPA_STATUS_SUCCESS) {
      *out = instHandles[0];
    }
  } else {
    PRINT_ERR("Error while getting a crypto instance.\n");
  }
  return (int) status;
}

int qat_cy_start_instance(CpaInstanceHandle inst) {
  return (int)cpaCyStartInstance(inst);
}

int qat_cy_stop_instance(CpaInstanceHandle inst) {
  return (int)cpaCyStopInstance(inst);
}

int qat_eddsa_sign(CpaInstanceHandle inst, Cpa8U *private_key,
                   Cpa8U *message_hash, Cpa8U *signature) {
  CpaStatus status = edDsaSign(private_key, message_hash, signature, inst);
  return (int)status;
}

int qat_set_address_translation(CpaInstanceHandle inst) {
  CpaStatus status = cpaCySetAddressTranslation(inst, qaeVirtToPhysNUMA);
  if (status != CPA_STATUS_SUCCESS) {
    PRINT_ERR("Error setting address translation for QAT instance\n");
  }
  return (int)status;
}

