#pragma once
#include <qat/cpa_eddsa_sample.h>
#include "icp_sal_user.h"

#ifdef __cplusplus
extern "C" {
#endif

int qat_start_session(const char *pProcessName);
int qat_stop_session();
int qat_get_instance(CpaInstanceHandle* out);
int qat_cy_start_instance(CpaInstanceHandle inst);
int qat_cy_stop_instance(CpaInstanceHandle inst);
int qat_eddsa_sign(CpaInstanceHandle inst, Cpa8U *private_key,
                   Cpa8U *message_hash, Cpa8U *signature);

#ifdef __cplusplus
}
#endif
